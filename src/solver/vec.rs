use crate::complex::*;
use crate::coord::{Coords, Point};
use crate::solver::{MbState, Solver};
use crate::threads::{Join, Split};

use super::D2ArrayLike;

#[derive(Clone, Debug)]
pub struct VecCell {
    pub(crate) c: C<f64>,
    pub(crate) z: C<f64>,
    pub(crate) i: i16,
}

#[derive(Clone, Debug)]
pub struct VecState {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) state: Vec<VecCell>,
}

impl From<Coords<C<f64>>> for VecState {
    fn from(v: Coords<C<f64>>) -> Self {
        let state: Vec<VecCell> = v
            .values
            .into_iter()
            .map(|c| VecCell { c, z: c, i: -1 })
            .collect();
        Self {
            width: v.width,
            height: v.height,
            state,
        }
    }
}

impl MbState for VecState {
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

impl Split for VecState {
    fn split_to_vec(self, n: usize) -> Vec<Self> {
        let rows = self.state.split_to_vec(self.height);
        let row_groups = rows.split_to_vec(n);

        let mut parts = vec![];
        for row_group in row_groups {
            let height = row_group.len();
            let state = Vec::<VecCell>::join_vec(row_group);
            parts.push(Self {
                width: self.width,
                height,
                state,
            })
        }
        parts
    }
}

impl Join for VecState {
    fn join_vec(parts: Vec<Self>) -> Self {
        let mut height = 0;
        let width = parts[0].width;
        let mut state_parts: Vec<Vec<VecCell>> = vec![];
        for part in parts {
            assert!(part.width == width);
            height += part.height;
            state_parts.push(part.state.clone());
        }
        Self {
            width,
            height,
            state: Vec::join_vec(state_parts),
        }
    }
}

impl D2ArrayLike for VecState {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            state: vec![
                VecCell {
                    c: cr(0.0),
                    z: cr(0.0),
                    i: 0
                };
                width * height
            ],
        }
    }
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn copy_from(&mut self, other: &Self, from: Point<usize>, to: Point<usize>) {
        self.state[to.row_idx(self.width)] = other.state[from.row_idx(other.width)].clone();
    }
    fn copy_self(&mut self, from: Point<usize>, to: Point<usize>) {
        self.state[to.row_idx(self.width)] = self.state[from.row_idx(self.width)].clone();
    }
}

#[derive(Clone)]
pub struct VecSolver {
    iterations: u16,
    treshold: f64,
}

impl Solver<VecState> for VecSolver {
    fn solve(&self, mut state: VecState) -> VecState {
        for iteration in 0..self.iterations {
            for cell in &mut state.state {
                if cell.i == -1 {
                    cell.z = (cell.z * cell.z) + cell.c;
                    if cell.z.norm() > self.treshold {
                        cell.i = iteration as i16;
                    }
                }
            }
        }
        state
    }
}

impl Default for VecSolver {
    fn default() -> Self {
        Self {
            iterations: 100,
            treshold: 2.0,
        }
    }
}
