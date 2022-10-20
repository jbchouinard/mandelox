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

pub trait Shift {
    fn shift_rows(&mut self, n: i64);
    fn shift_cols(&mut self, n: i64);
}

pub trait Slice {
    fn slice_rows(&self, n: usize) -> &Self;
    fn slice_cols(&self, n: usize) -> &Self;
}

pub trait MbState {
    fn initialize(width: usize, height: usize, grid: &Viewport<f64>) -> Self;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn i_value(&self, x: usize, y: usize) -> i16;
}

pub fn default_solver() -> WorkerPool<MbVecState, MbVecState> {
    MbVecSolver::default().threaded(num_cpus::get_physical())
}

// pub struct Mandelbrot<S, T> {
//     pub viewport: Viewport<f64>,
//     pub state: T,
//     pub solver: S,
// }
