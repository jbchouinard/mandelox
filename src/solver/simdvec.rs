use lazy_static::lazy_static;
use ultraviolet::{f64x4, DMat2x4, DVec2x4};
use wide::CmpGt;

use crate::complex::C;
use crate::{coord::Coords, Join, MbState, Solver, Split};

lazy_static! {
    static ref INF: f64x4 = f64x4::splat(f64::INFINITY);
    static ref ZERO: f64x4 = f64x4::splat(0.0);
    static ref ONE: f64x4 = f64x4::splat(1.0);
}

pub type C4 = DMat2x4;

pub fn c4(re: f64x4, im: f64x4) -> C4 {
    DMat2x4::new(DVec2x4::new(re, im), DVec2x4::new(-im, re))
}

pub fn cr4(re: f64x4) -> C4 {
    c4(re, f64x4::splat(0.0))
}

pub fn ci4(im: f64x4) -> C4 {
    c4(f64x4::splat(0.0), im)
}

pub fn re4(c: C4) -> f64x4 {
    c.cols[0].x
}

pub fn im4(c: C4) -> f64x4 {
    c.cols[0].y
}

pub fn abs(c: C4) -> f64x4 {
    c.cols[0].mag()
}

pub fn c4tostr(c: C4) -> String {
    let re = re4(c).to_array();
    let im = im4(c).to_array();
    format!(
        "[{}{:+}j, {}{:+}j, {}{:+}j, {}{:+}j]",
        re[0], im[0], re[1], im[1], re[2], im[2], re[3], im[3]
    )
}

#[derive(Debug, Clone)]
pub struct SimdVecCell {
    pub(crate) c: C4,
    pub(crate) z: C4,
    pub(crate) i: f64x4,
}

impl SimdVecCell {
    pub fn iterate(&mut self, iterations: usize, mut iteration: f64x4, treshold: f64x4) {
        for _ in 0..iterations {
            iteration += *ONE;
            self.z = (self.z * self.z) + self.c;
            let z_abs = abs(self.z);
            let diverged = z_abs.cmp_gt(treshold);
            let diverged_i = diverged.blend(iteration, *INF);
            self.i = self.i.min(diverged_i);
        }
    }
}

#[derive(Clone)]
pub struct SimdVecState {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) state: Vec<SimdVecCell>,
}

impl MbState for SimdVecState {
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn i_value(&self, x: usize, y: usize) -> i16 {
        let n = self.width * y + x;
        let ival = self.state[n / 4].i.as_array_ref()[n % 4];
        if ival == f64::INFINITY {
            -1
        } else {
            ival as i16
        }
    }
}

impl From<Coords<C<f64>>> for SimdVecState {
    fn from(v: Coords<C<f64>>) -> Self {
        let cs: Vec<num::complex::Complex<f64>> = v.values;
        assert!(cs.len() % 4 == 0, "oops");
        let mut state = Vec::with_capacity(cs.len() / 4);

        for i in (0..cs.len()).step_by(4) {
            let re = [cs[i].re, cs[i + 1].re, cs[i + 2].re, cs[i + 3].re];
            let im = [cs[i].im, cs[i + 1].im, cs[i + 2].im, cs[i + 3].im];
            let c = c4(f64x4::new(re), f64x4::new(im));
            state.push(SimdVecCell {
                c,
                z: c,
                i: f64x4::splat(f64::INFINITY),
            })
        }

        Self {
            width: v.width,
            height: v.height,
            state,
        }
    }
}

impl Split for SimdVecState {
    fn split_to_vec(self, n: usize) -> Vec<Self> {
        let rows = self.state.split_to_vec(self.height);
        let row_groups = rows.split_to_vec(n);

        let mut parts = vec![];
        for row_group in row_groups {
            let height = row_group.len();
            let state = Vec::<SimdVecCell>::join_vec(row_group);
            parts.push(Self {
                width: self.width,
                height,
                state,
            })
        }
        parts
    }
}

impl Join for SimdVecState {
    fn join_vec(parts: Vec<Self>) -> Self {
        let mut height = 0;
        let width = parts[0].width;
        let mut state_parts: Vec<Vec<SimdVecCell>> = vec![];
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

#[derive(Clone, Debug)]
pub struct SimdVecSolver {
    treshold: f64x4,
}

impl Default for SimdVecSolver {
    fn default() -> Self {
        Self {
            treshold: f64x4::splat(2.0),
        }
    }
}

impl Solver<SimdVecState> for SimdVecSolver {
    fn solve(&self, mut state: SimdVecState) -> SimdVecState {
        for cell in &mut state.state {
            let mut iteration = *ZERO;
            for _ in 0..100 {
                iteration += *ONE;
                cell.z = (cell.z * cell.z) + cell.c;
                let z_abs = abs(cell.z);
                let diverged = z_abs.cmp_gt(self.treshold);
                let diverged_i = diverged.blend(iteration, *INF);
                cell.i = cell.i.min(diverged_i);
            }
        }
        state
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::coord::Viewbox;

    #[test]
    fn test_complex4() {
        let treshold = f64x4::splat(100.0);
        let SimdVecState { mut state, .. } =
            Viewbox::initial(8, 1).generate_complex_coordinates().into();
        let mut cell1 = state.pop().unwrap();
        let mut cell2 = state.pop().unwrap();
        // println!("{:?} {:?}", cell1.i, cell2.i);

        cell1.iterate(100, f64x4::splat(0.0), treshold);
        cell2.iterate(100, f64x4::splat(0.0), treshold);
        // println!("{:?} {:?}", cell1.i, cell2.i);
    }
}
