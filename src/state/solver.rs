use std::sync::{mpsc, Arc};
use std::thread;

use ndarray::{Array2, Zip};

use super::MbState;

pub trait IterSolver {
    fn iterate_n(&self, state: &MbState, n: u16) -> MbState;
}

#[derive(Clone)]
pub struct MbSolver {
    treshold: f64,
}

impl MbSolver {
    pub fn new(treshold: f64) -> Self {
        Self { treshold }
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

impl IterSolver for MbSolver {
    fn iterate_n(&self, state: &MbState, n: u16) -> MbState {
        let mut state = state.clone();
        for _ in 0..n {
            state = self.iterate(&state);
        }
        state
    }
}

#[derive(Clone)]
struct MbStateSegment {
    n: usize,
    state: MbState,
}

pub struct ThreadedMbSolver {
    treshold: f64,
    threads: usize,
}

impl ThreadedMbSolver {
    pub fn new(treshold: f64, threads: usize) -> Self {
        if threads < 1 {
            panic!("need at least 1 thread");
        }
        Self { treshold, threads }
    }
}

impl IterSolver for ThreadedMbSolver {
    fn iterate_n(&self, state: &MbState, n: u16) -> MbState {
        let states = state.split(self.threads);
        let (tx, rx) = mpsc::channel();

        let mut handles = vec![];
        for (sn, state) in states.into_iter().enumerate() {
            let txi = tx.clone();
            let treshold = self.treshold;
            let handle = thread::spawn(move || {
                let solver = MbSolver::new(treshold);
                let solved = solver.iterate_n(&state, n);
                txi.send(MbStateSegment {
                    n: sn,
                    state: solved,
                })
                .unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut opt_solved_states: Vec<Option<MbState>> = vec![None; self.threads];
        for _ in 0..self.threads {
            let segment: MbStateSegment = rx.try_recv().unwrap();
            opt_solved_states[segment.n] = Some(segment.state);
        }

        let mut solved_states: Vec<MbState> = vec![];
        for opt_state in opt_solved_states {
            match opt_state {
                Some(state) => solved_states.push(state),
                None => panic!("missing state segment"),
            }
        }

        solved_states[0].join(&solved_states[1..])
    }
}
