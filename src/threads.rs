use std::iter::zip;
use std::sync::mpsc;
use std::thread;

pub trait Call<T, U> {
    fn call(&self, t: T) -> U;
}

impl<F, T, U> Call<T, U> for F
where
    F: Fn(T) -> U,
{
    fn call(&self, t: T) -> U {
        self(t)
    }
}

pub trait Threaded<T, U>
where
    T: Split,
    U: Join,
{
    fn threadpool(&self, n: usize) -> WorkerPool<T, U>;
}

impl<C, T, U> Threaded<T, U> for C
where
    C: Call<T, U> + Clone + Send + 'static,
    T: Split + Send + 'static,
    U: Join + Send + 'static,
{
    fn threadpool(&self, n: usize) -> WorkerPool<T, U> {
        WorkerPool::with_cloned(n, self)
    }
}

pub trait Split: Sized {
    fn split_to_vec(self, n: usize) -> Vec<Self>;

    fn to_parts(self, n: usize) -> Vec<SplitPart<Self>> {
        self.split_to_vec(n)
            .into_iter()
            .enumerate()
            .map(|(n, part)| SplitPart::new(part, n))
            .collect()
    }
}

pub fn vectorize<F, T, U>(f: F) -> impl Call<Vec<T>, Vec<U>> + Send + Clone + 'static
where
    F: Call<T, U> + Send + Clone + 'static,
{
    move |vt: Vec<T>| vt.into_iter().map(|t| f.call(t)).collect()
}

pub trait Join: Sized {
    fn join_vec(parts: Vec<Self>) -> Self;
}

pub struct RangeSplitter {
    i: usize,
    n: usize,
    offset: usize,
    length: usize,
    size: usize,
    plus_ones: usize,
}

impl RangeSplitter {
    pub fn split(start: usize, end: usize, n: usize) -> Self {
        Self {
            i: 0,
            n,
            offset: start,
            length: end - start,
            size: (end - start) / n,
            plus_ones: (end - start) % n,
        }
    }
}

impl Iterator for RangeSplitter {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<(usize, usize)> {
        if self.i >= self.n {
            return None;
        }
        self.i += 1;
        let mut size = self.size;
        if self.plus_ones > 0 {
            size += 1;
            self.plus_ones -= 1;
        }
        assert!(size <= self.length, "RangeSplitter bounds error");
        let range = (self.offset, self.offset + size);
        self.offset += size;
        self.length -= size;
        Some(range)
    }
}

impl<T> Split for Vec<T>
where
    T: Clone,
{
    fn split_to_vec(self, n: usize) -> Vec<Self> {
        RangeSplitter::split(0, self.len(), n)
            .map(|(i, j)| self[i..j].to_vec())
            .collect()
    }
}

impl<T> Join for Vec<T> {
    fn join_vec(parts: Vec<Self>) -> Self {
        let mut v: Vec<T> = vec![];
        for p in parts {
            v.extend(p);
        }
        v
    }
}

#[derive(Debug, Clone)]
pub struct JoinError;

#[derive(Debug)]
pub struct SplitPart<T> {
    n: usize,
    part: T,
}

impl<T> SplitPart<T> {
    pub fn new(part: T, n: usize) -> Self {
        Self { part, n }
    }
}

impl<T> SplitPart<T>
where
    T: Join,
{
    pub fn join(splits: Vec<SplitPart<T>>) -> Result<T, JoinError> {
        let n = splits.len();
        if n == 0 {
            return Err(JoinError);
        }
        let mut parts: Vec<Option<T>> = (0..n).map(|_| None).collect();
        for s in splits {
            if s.n >= n {
                return Err(JoinError);
            }
            if parts[s.n].is_some() {
                return Err(JoinError);
            }
            parts[s.n] = Some(s.part);
        }
        // By pigeonhole principle, no element can be None
        let parts: Vec<T> = parts.into_iter().map(|x| x.unwrap()).collect();
        Ok(T::join_vec(parts))
    }
}

struct Worker<T> {
    param_tx: mpsc::Sender<SplitPart<T>>,
}

impl<T> Worker<T>
where
    T: Split,
{
    fn new<F, U>(f: F, return_tx: mpsc::Sender<SplitPart<U>>) -> Self
    where
        F: Call<T, U> + Send + 'static,
        T: Split + Send + 'static,
        U: Join + Send + 'static,
    {
        let (param_tx, param_rx) = mpsc::channel::<SplitPart<T>>();

        thread::spawn(move || loop {
            let splitted = match param_rx.recv() {
                Ok(s) => s,
                Err(_) => return,
            };
            let res = f.call(splitted.part);
            if return_tx.send(SplitPart::new(res, splitted.n)).is_err() {
                return;
            }
        });

        Self { param_tx }
    }

    fn send(&self, part: SplitPart<T>) {
        self.param_tx.send(part).unwrap();
    }
}

pub struct WorkerPool<T, U>
where
    T: Split,
    U: Join,
{
    workers: Vec<Worker<T>>,
    tx: mpsc::Sender<SplitPart<U>>,
    rx: mpsc::Receiver<SplitPart<U>>,
}

impl<T, U> WorkerPool<T, U>
where
    T: Split + Send + 'static,
    U: Join + Send + 'static,
{
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            workers: vec![],
            rx,
            tx,
        }
    }

    fn add_workers<F, G>(&mut self, n: usize, g: G)
    where
        F: Call<T, U> + Send + 'static,
        G: Fn() -> F,
    {
        for _ in 0..n {
            self.workers.push(Worker::new(g(), self.tx.clone()));
        }
    }

    pub fn with<F, G>(n: usize, factory: G) -> Self
    where
        F: Call<T, U> + Send + 'static,
        G: Fn() -> F,
    {
        let mut this = Self::new();
        this.add_workers(n, factory);
        this
    }

    pub fn with_cloned<F>(n: usize, f: &F) -> Self
    where
        F: Call<T, U> + Send + Clone + 'static,
    {
        Self::with(n, || f.clone())
    }
}

impl<T, U> Call<T, U> for WorkerPool<T, U>
where
    T: Split,
    U: Join,
{
    fn call(&self, t: T) -> U {
        let sn = self.workers.len();
        assert!(sn > 0, "no workers");

        for (worker, part) in zip(&self.workers, t.to_parts(sn)) {
            worker.send(part);
        }
        let mut parts: Vec<SplitPart<U>> = vec![];
        for _ in 0..sn {
            parts.push(self.rx.recv().unwrap());
        }
        SplitPart::join(parts).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::{vectorize, Call, Split, SplitPart, Threaded};

    fn test_vec_split(length: usize, n: usize) {
        let v: Vec<usize> = (0..length).collect();
        let vs = v.clone().to_parts(n);
        assert_eq!(vs.len(), n);
        let vj: Vec<usize> = SplitPart::join(vs).unwrap();
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

    fn mul2(x: i64) -> i64 {
        2 * x
    }

    #[test]
    fn test_worker_pool() {
        let q = || (0..10).collect::<Vec<i64>>();
        let f = vectorize(mul2);
        let res = f.call(q());

        assert_eq!(res, f.threadpool(1).call(q()));
        assert_eq!(res, f.threadpool(5).call(q()));
        assert_eq!(res, f.threadpool(10).call(q()));
        assert_eq!(res, f.threadpool(20).call(q()));
    }
}
