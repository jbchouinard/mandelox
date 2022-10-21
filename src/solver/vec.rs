use crate::complex::*;
use crate::coord::Viewbox;
use crate::solver::{MbState, Solver};
use crate::threads::{Join, Split};

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
    pub(crate) iteration: i16,
    pub(crate) state: Vec<VecCell>,
}

impl From<Viewbox> for VecState {
    fn from(v: Viewbox) -> Self {
        let state: Vec<VecCell> = v
            .generate_complex_coordinates()
            .into_iter()
            .map(|c| VecCell { c, z: c, i: -1 })
            .collect();
        Self {
            width: v.width as usize,
            height: v.height as usize,
            iteration: 0,
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
                iteration: self.iteration,
            })
        }
        parts
    }
}

impl Join for VecState {
    fn join_vec(parts: Vec<Self>) -> Self {
        let mut height = 0;
        let width = parts[0].width;
        let iteration = parts[0].iteration;
        let mut state_parts: Vec<Vec<VecCell>> = vec![];
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
        state.iteration = self.iterations as i16;
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
