use crate::coord::Viewport;
use crate::threads::{Join, Split, WorkerPool};

pub mod array;
pub mod cell;
pub mod sarray;
pub mod vec;

pub use array::{MbArraySolver, MbArrayState};
pub use cell::{MbCellSolver, MbCellState};
pub use vec::{MbVecSolver, MbVecState};

pub trait Solver<T> {
    fn solve(&self, state: T) -> T;

    fn threaded(self, n: usize) -> WorkerPool<T, T>
    where
        Self: Clone + Send + 'static,
        T: Split + Join + Send + 'static,
    {
        WorkerPool::with(n, || {
            let solver = self.clone();
            move |state| solver.solve(state)
        })
    }
}

pub trait MbState {
    fn initialize(width: usize, height: usize, grid: &Viewport) -> Self;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn i_value(&self, x: usize, y: usize) -> i16;
}
