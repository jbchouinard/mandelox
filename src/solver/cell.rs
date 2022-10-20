use std::iter::zip;

use crate::complex::{ci, cr, C};
use crate::coord::Viewport;
use crate::solver::sarray::{SArray, SArrayLen, SArraySplit};
use crate::solver::{MbState, Solver};
use crate::threads::{Join, Split};

pub const MB_CELL_STATE_WIDTH: usize = 75;
pub const MB_CELL_STATE_HEIGHT: usize = 50;
const ARRAY_L: usize = MB_CELL_STATE_HEIGHT * MB_CELL_STATE_WIDTH;

pub type CArray = SArray<C<f64>, ARRAY_L>;
pub type IArray = SArray<i16, ARRAY_L>;

pub struct MbCellState {
    iteration: i16,
    c: CArray,
    z: CArray,
    i: IArray,
}

impl Split for MbCellState {
    fn split_to_vec(self, n: usize) -> Vec<Self> {
        let Self { iteration, c, z, i } = self;
        zip(zip(c.split(n), z.split(n)), i.split(n))
            .map(|((c, z), i)| Self { iteration, c, z, i })
            .collect()
    }
}

impl Join for MbCellState {
    fn join_vec(parts: Vec<Self>) -> Self {
        let iteration = parts[0].iteration;
        let (czs, is): (Vec<(CArray, CArray)>, Vec<IArray>) = parts
            .into_iter()
            .map(|cell| {
                let Self { c, z, i, .. } = cell;
                ((c, z), i)
            })
            .unzip();
        let (cs, zs): (Vec<CArray>, Vec<CArray>) = czs.into_iter().unzip();
        let c = SArray::join(cs);
        let z = SArray::join(zs);
        let i = SArray::join(is);
        Self { iteration, c, z, i }
    }
}

impl MbState for MbCellState {
    fn initialize(width: usize, height: usize, grid: &Viewport<f64>) -> Self {
        assert!(width == MB_CELL_STATE_WIDTH, "wrong width");
        assert!(height == MB_CELL_STATE_HEIGHT, "wrong height");
        let x_b = cr(grid.x.min);
        let x_m = cr(grid.x.length() / (width as f64 - 1.0));
        let y_b = cr(grid.y.min);
        let y_m = cr(grid.y.length() / (height as f64 - 1.0));

        let mut c = CArray::default();
        let mut z = CArray::default();
        let i = IArray::full(-1);

        for y in 0..MB_CELL_STATE_HEIGHT {
            for x in 0..MB_CELL_STATE_WIDTH {
                let j = (y * width) + x;
                let cx = x_b + x_m * x as f64;
                let cy = y_b + y_m * y as f64;
                let cc = cx + cy * ci(1.0);
                c.set(j, cc);
                z.set(j, cc);
            }
        }

        Self {
            iteration: 0,
            c,
            z,
            i,
        }
    }
    fn height(&self) -> usize {
        MB_CELL_STATE_HEIGHT
    }
    fn width(&self) -> usize {
        MB_CELL_STATE_WIDTH
    }
    fn i_value(&self, x: usize, y: usize) -> i16 {
        self.i.get(y * MB_CELL_STATE_WIDTH + x)
    }
}

pub struct MbCellSolver {
    iterations: i16,
    treshold: f64,
}

impl Default for MbCellSolver {
    fn default() -> Self {
        Self {
            iterations: 100,
            treshold: 2.0,
        }
    }
}

impl MbCellSolver {
    fn iterate(&self, state: &mut MbCellState) {
        state.iteration += 1;
        for j in 0..state.c.len() {
            let c = state.c.get(j);
            let z = state.z.get(j);
            let i = state.i.get(j);
            let z = (z * z) + c;
            state.z.set(j, z);
            if (i == -1) && (z.norm() > self.treshold) {
                state.i.set(j, state.iteration);
            }
        }
    }
}

impl Solver<MbCellState> for MbCellSolver {
    fn solve(&self, mut state: MbCellState) -> MbCellState {
        for _ in 0..self.iterations {
            self.iterate(&mut state);
        }
        state
    }
}
