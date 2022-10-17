use std::iter::zip;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use ndarray::{Array2, Zip};

use super::MbState;

pub trait MbSolver {
    fn solve(&self, state: &MbState) -> MbState;
}

#[derive(Clone)]
pub struct IterSolver {
    iterations: u16,
    treshold: f64,
}

impl IterSolver {
    pub fn new(treshold: f64, iterations: u16) -> Self {
        Self {
            treshold,
            iterations,
        }
    }

    fn iterate(&self, state: &MbState) -> MbState {
        let mut new_za = Array2::zeros((state.height, state.width));
        let mut new_ia = Array2::zeros((state.height, state.width));

        Zip::from(state.ia.as_ref())
            .and(&mut new_ia)
            .and(state.za.as_ref())
            .and(&mut new_za)
            .and(state.ca.as_ref())
            .for_each(|&iv, niv, &zv, nzv, &cv| {
                *nzv = (zv * zv) + cv;
                *niv = if (iv == -1) && (nzv.norm() > self.treshold) {
                    state.iteration + 1
                } else {
                    iv
                };
            });

        MbState {
            height: state.height(),
            width: state.width(),
            iteration: state.iteration + 1,
            ca: state.ca.clone(),
            za: Arc::new(new_za),
            ia: Arc::new(new_ia),
        }
    }
}

impl Default for IterSolver {
    fn default() -> Self {
        Self::new(2.0, 100)
    }
}

impl MbSolver for IterSolver {
    fn solve(&self, state: &MbState) -> MbState {
        let mut state = state.clone();
        for _ in 0..self.iterations {
            state = self.iterate(&state);
        }
        state
    }
}

#[derive(Clone)]
struct MbStateSegment {
    pub n: usize,
    pub state: MbState,
}

impl MbStateSegment {
    pub fn new(n: usize, state: MbState) -> Self {
        Self { n, state }
    }
}

struct MbWorker {
    tx: mpsc::Sender<MbStateSegment>,
}

impl MbWorker {
    fn new<T>(solver: T, sol_tx: mpsc::Sender<MbStateSegment>) -> Self
    where
        T: MbSolver + Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<MbStateSegment>();
        thread::spawn(move || loop {
            println!("recv problem");
            let recv_segment = match rx.recv() {
                Ok(segment) => segment,
                Err(_) => return,
            };
            let soln = solver.solve(&recv_segment.state);
            println!("send soln");
            sol_tx
                .send(MbStateSegment {
                    n: recv_segment.n,
                    state: soln,
                })
                .unwrap();
        });

        Self { tx }
    }

    fn send(&self, segment: MbStateSegment) {
        println!("send problem");
        self.tx.send(segment).unwrap();
    }
}

pub struct MultiSolver {
    workers: Vec<MbWorker>,
    rx: mpsc::Receiver<MbStateSegment>,
    tx: mpsc::Sender<MbStateSegment>,
}

impl MultiSolver {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            workers: vec![],
            rx,
            tx,
        }
    }

    pub fn add_solver<T>(&mut self, solver: T)
    where
        T: MbSolver + Send + 'static,
    {
        let worker = MbWorker::new(solver, self.tx.clone());
        self.workers.push(worker);
    }

    pub fn add_default_solvers<T>(&mut self, n: usize)
    where
        T: MbSolver + Send + 'static + Default,
    {
        for _ in 0..n {
            self.add_solver(T::default())
        }
    }

    pub fn with_default_solvers<T>(n: usize) -> Self
    where
        T: MbSolver + Send + 'static + Default,
    {
        let mut this = Self::new();
        this.add_default_solvers::<T>(n);
        this
    }

    pub fn add_cloned_solvers<T>(&mut self, n: usize, solver: &T)
    where
        T: MbSolver + Send + 'static + Clone,
    {
        for _ in 0..n {
            self.add_solver(solver.clone());
        }
    }

    pub fn with_cloned_solvers<T>(n: usize, solver: &T) -> Self
    where
        T: MbSolver + Send + 'static + Clone,
    {
        let mut this = Self::new();
        this.add_cloned_solvers(n, solver);
        this
    }
}

impl Default for MultiSolver {
    fn default() -> Self {
        Self::with_default_solvers::<IterSolver>(4)
    }
}

impl MbSolver for MultiSolver {
    fn solve(self: &MultiSolver, state: &MbState) -> MbState {
        let sn = self.workers.len();
        assert!(sn > 0, "no workers");

        for (worker, (n, state)) in zip(&self.workers, state.split(sn).into_iter().enumerate()) {
            worker.send(MbStateSegment::new(n, state));
        }

        let mut soln_segments: Vec<Option<MbState>> = vec![None; sn];
        for _ in 0..sn {
            println!("recv soln");
            let segment = self.rx.recv().unwrap();
            soln_segments[segment.n] = Some(segment.state);
        }

        let soln_segments: Vec<MbState> = soln_segments
            .into_iter()
            .map(|maybe_state| maybe_state.expect("missing solution"))
            .collect();

        soln_segments[0].join(&soln_segments[1..])
    }
}
