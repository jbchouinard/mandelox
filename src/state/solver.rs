use std::sync::Arc;

use ndarray::{Array2, Zip};

use super::{MbArrayState, MbVecCell, MbVecState};
use crate::threads::Solver;

#[derive(Clone)]
pub struct MbVecSolver {
    iterations: u16,
    treshold: f64,
}

impl MbVecSolver {
    fn iterate(&self, state: &MbVecState) -> MbVecState {
        let mut new_state: Vec<MbVecCell> = Vec::with_capacity(state.width * state.height);
        let iteration = state.iteration + 1;

        for cell in &state.state {
            let c = cell.c;
            let z = (cell.z * cell.z) + cell.c;
            let i = if (cell.i == -1) && (z.norm() > self.treshold) {
                iteration
            } else {
                cell.i
            };

            new_state.push(MbVecCell { c, z, i })
        }

        MbVecState {
            width: state.width,
            height: state.height,
            iteration,
            state: new_state,
        }
    }
}

impl Solver<MbVecState> for MbVecSolver {
    fn solve(&self, state: MbVecState) -> MbVecState {
        let mut state = state.clone();
        for _ in 0..self.iterations {
            state = self.iterate(&state);
        }
        state
    }
}

impl Default for MbVecSolver {
    fn default() -> Self {
        Self {
            iterations: 100,
            treshold: 2.0,
        }
    }
}

#[derive(Clone)]
pub struct MbArraySolver {
    iterations: u16,
    treshold: f64,
}

impl MbArraySolver {
    pub fn new(treshold: f64, iterations: u16) -> Self {
        Self {
            treshold,
            iterations,
        }
    }

    fn iterate(&self, state: &MbArrayState) -> MbArrayState {
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

        MbArrayState {
            height: state.height(),
            width: state.width(),
            iteration: state.iteration + 1,
            ca: state.ca.clone(),
            za: Arc::new(new_za),
            ia: Arc::new(new_ia),
        }
    }
}

impl Default for MbArraySolver {
    fn default() -> Self {
        Self::new(2.0, 100)
    }
}

impl Solver<MbArrayState> for MbArraySolver {
    fn solve(&self, state: MbArrayState) -> MbArrayState {
        let mut state = state.clone();
        for _ in 0..self.iterations {
            state = self.iterate(&state);
        }
        state
    }
}
