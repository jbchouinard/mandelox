use std::iter::zip;
use std::sync::mpsc;
use std::thread;

pub trait Split: Sized + Clone {
    fn split_parts(&self, n: usize) -> Vec<Self>;

    fn join_parts(&self, parts: &[Self]) -> Self;

    fn parts(&self, n: usize) -> Vec<SplitPart<Self>> {
        self.split_parts(n)
            .into_iter()
            .enumerate()
            .map(|(n, part)| SplitPart::new(part, n))
            .collect()
    }
}

impl<T> Split for Vec<T>
where
    T: Clone,
{
    fn split_parts(&self, n: usize) -> Vec<Self> {
        let size = self.len() / n;
        let size_xtra = self.len() % n;

        let mut start = 0;
        let mut end = size;
        let mut parts: Vec<Vec<T>> = vec![];
        for i in 0..n {
            if i < size_xtra {
                end += 1
            }
            parts.push(self[start..end].to_vec());
            start = end;
            end += size;
        }
        parts
    }
    fn join_parts(&self, parts: &[Self]) -> Self {
        let mut v: Vec<T> = self.clone();
        for p in parts {
            v.extend_from_slice(p);
        }
        v
    }
}

#[derive(Debug, Clone)]
pub struct JoinError;

#[derive(Debug)]
pub struct SplitPart<T: Split> {
    pub n: usize,
    pub part: T,
}

impl<T> SplitPart<T>
where
    T: Split,
{
    pub fn new(part: T, n: usize) -> Self {
        Self { part, n }
    }

    pub fn join(splits: &[SplitPart<T>]) -> Result<T, JoinError> {
        let n = splits.len();
        if n == 0 {
            return Err(JoinError);
        }
        let mut parts: Vec<Option<T>> = vec![None; n];
        for s in splits {
            if s.n >= n {
                return Err(JoinError);
            }
            if parts[s.n].is_some() {
                return Err(JoinError);
            }
            parts[s.n] = Some(s.part.clone());
        }
        // By pigeonhole principle, no elements can be None
        let parts: Vec<T> = parts.into_iter().map(|x| x.unwrap()).collect();
        Ok(parts[0].join_parts(&parts[1..]))
    }
}

pub trait Solver<T> {
    fn solve(&self, state: &T) -> T;
}

pub trait Threaded<T>
where
    T: Split,
{
    fn threaded(&self, n: usize) -> ThreadedSolver<T>;
}

impl<S, T> Threaded<T> for S
where
    T: Split + Send + 'static,
    S: Solver<T> + Send + 'static + Clone,
{
    fn threaded(&self, n: usize) -> ThreadedSolver<T> {
        ThreadedSolver::with_cloned_solvers(n, self)
    }
}

pub trait DefaultThreaded<T>
where
    T: Split,
{
    fn threaded(n: usize) -> ThreadedSolver<T>;
}

impl<S, T> DefaultThreaded<T> for S
where
    T: Split + Send + 'static,
    S: Solver<T> + Send + 'static + Default,
{
    fn threaded(n: usize) -> ThreadedSolver<T> {
        ThreadedSolver::with_default_solvers::<S>(n)
    }
}

pub fn make_solver<S, T>(threads: usize) -> Box<dyn Solver<T>>
where
    T: Split + Send + 'static,
    S: Solver<T> + Send + 'static + Clone + Default,
{
    if threads == 0 {
        Box::<S>::default()
    } else {
        Box::new(S::default().threaded(threads))
    }
}

struct Worker<T>
where
    T: Split,
{
    tx: mpsc::Sender<SplitPart<T>>,
}

impl<T> Worker<T>
where
    T: Split,
{
    fn new<S>(solver: S, sol_tx: mpsc::Sender<SplitPart<T>>) -> Self
    where
        S: Solver<T> + Send + 'static,
        T: Split + Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<SplitPart<T>>();
        thread::spawn(move || loop {
            let splitted = match rx.recv() {
                Ok(s) => s,
                Err(_) => return,
            };
            let soln = solver.solve(&splitted.part);
            sol_tx.send(SplitPart::new(soln, splitted.n)).unwrap();
        });

        Self { tx }
    }

    fn send(&self, part: SplitPart<T>) {
        self.tx.send(part).unwrap();
    }
}

pub struct ThreadedSolver<T>
where
    T: Split,
{
    workers: Vec<Worker<T>>,
    rx: mpsc::Receiver<SplitPart<T>>,
    tx: mpsc::Sender<SplitPart<T>>,
}

impl<T> ThreadedSolver<T>
where
    T: Split + Send + 'static,
{
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            workers: vec![],
            rx,
            tx,
        }
    }

    pub fn add_solver<S>(&mut self, solver: S)
    where
        S: Solver<T> + Send + 'static,
    {
        let worker = Worker::new(solver, self.tx.clone());
        self.workers.push(worker);
    }

    pub fn with_solvers<F, S>(n: usize, f: F) -> Self
    where
        S: Solver<T> + Send + 'static,
        F: Fn() -> S,
    {
        let mut this = Self::new();
        for _ in 0..n {
            this.add_solver(f());
        }
        this
    }

    pub fn add_default_solvers<S>(&mut self, n: usize)
    where
        S: Solver<T> + Send + 'static + Default,
    {
        for _ in 0..n {
            self.add_solver(S::default())
        }
    }

    pub fn with_default_solvers<S>(n: usize) -> Self
    where
        S: Solver<T> + Send + 'static + Default,
    {
        let mut this = Self::new();
        this.add_default_solvers::<S>(n);
        this
    }

    pub fn add_cloned_solvers<S>(&mut self, n: usize, solver: &S)
    where
        S: Solver<T> + Send + 'static + Clone,
    {
        for _ in 0..n {
            self.add_solver(solver.clone());
        }
    }

    pub fn with_cloned_solvers<S>(n: usize, solver: &S) -> Self
    where
        S: Solver<T> + Send + 'static + Clone,
    {
        let mut this = Self::new();
        this.add_cloned_solvers(n, solver);
        this
    }
}

impl<T> Solver<T> for ThreadedSolver<T>
where
    T: Split,
{
    fn solve(&self, state: &T) -> T {
        let sn = self.workers.len();
        assert!(sn > 0, "no workers");

        for (worker, part) in zip(&self.workers, state.parts(sn)) {
            worker.send(part);
        }
        let mut parts: Vec<SplitPart<T>> = vec![];
        for _ in 0..sn {
            parts.push(self.rx.recv().unwrap());
        }
        SplitPart::join(&parts).unwrap()
    }
}

#[cfg(test)]
fn test_vec_split(length: usize, n: usize) {
    let v: Vec<usize> = (0..length).collect();
    let vs = v.parts(n);
    assert_eq!(vs.len(), n);
    let vj: Vec<usize> = SplitPart::join(&vs).unwrap();
    assert_eq!(v, vj);
}

#[test]
fn test_vec_splits() {
    test_vec_split(1, 1);
    test_vec_split(0, 2);
    test_vec_split(5, 8);
    test_vec_split(8, 5);
    test_vec_split(100, 1);
    test_vec_split(55, 47);
}
