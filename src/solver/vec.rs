use druid::Data;

use crate::complex::*;
use crate::coord::Viewport;
use crate::solver::{MbState, Solver};
use crate::threads::{Join, Split};

#[derive(Clone, Debug)]
pub struct MbVecCell {
    c: C<f64>,
    z: C<f64>,
    i: i16,
}

#[derive(Clone, Debug)]
pub struct MbVecState {
    width: usize,
    height: usize,
    iteration: i16,
    state: Vec<MbVecCell>,
}

impl Data for MbVecState {
    fn same(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.iteration == other.iteration
            && self.state[self.width + 2].c == other.state[self.width + 2].c
    }
}

impl MbState for MbVecState {
    fn initialize(width: usize, height: usize, grid: &Viewport<f64>) -> Self {
        let x_b = cr(grid.x.min);
        let x_m = cr(grid.x.length() / (width as f64 - 1.0));
        let y_b = ci(grid.y.min);
        let y_m = ci(grid.y.length() / (height as f64 - 1.0));

        let mut state = Vec::with_capacity(width * height);
        let mut cy = y_b;
        for _ in 0..height {
            let mut cx = x_b;
            for _ in 0..width {
                let c = cx + cy;
                state.push(MbVecCell { c, z: c, i: -1 });
                cx += x_m;
            }
            cy += y_m;
        }

        Self {
            width,
            height,
            state,
            iteration: 0,
        }
    }
    fn height(&self) -> usize {
        self.height
    }
    fn width(&self) -> usize {
        self.width
    }
    fn i_value(&self, x: usize, y: usize) -> i16 {
        self.state[y * self.width + x].i
    }
}

impl Split for MbVecState {
    fn split_to_vec(self, n: usize) -> Vec<Self> {
        let rows = self.state.split_to_vec(self.height);
        let row_groups = rows.split_to_vec(n);

        let mut parts = vec![];
        for row_group in row_groups {
            let height = row_group.len();
            let state = Vec::<MbVecCell>::join_vec(row_group);
            parts.push(Self {
                width: self.width,
                height,
                state,
                iteration: self.iteration,
            })
        }
        parts
    }
}

impl Join for MbVecState {
    fn join_vec(parts: Vec<Self>) -> Self {
        let mut height = 0;
        let width = parts[0].width;
        let iteration = parts[0].iteration;
        let mut state_parts: Vec<Vec<MbVecCell>> = vec![];
        for part in parts {
            assert!(part.width == width);
            assert!(part.iteration == iteration);
            height += part.height;
            state_parts.push(part.state.clone());
        }
        Self {
            width,
            height,
            iteration,
            state: Vec::join_vec(state_parts),
        }
    }
}

#[derive(Clone)]
pub struct MbVecSolver {
    iterations: u16,
    treshold: f64,
}

impl MbVecSolver {
    fn iterate(&self, state: &mut MbVecState) {
        state.iteration += 1;
        for cell in &mut state.state {
            cell.z = (cell.z * cell.z) + cell.c;
            if (cell.i == -1) && (cell.z.norm() > self.treshold) {
                cell.i = state.iteration
            }
        }
    }
}

impl Solver<MbVecState> for MbVecSolver {
    fn solve(&self, mut state: MbVecState) -> MbVecState {
        for _ in 0..self.iterations {
            self.iterate(&mut state);
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
