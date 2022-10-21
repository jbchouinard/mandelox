use lazy_static::lazy_static;
use ultraviolet::{f64x4, DMat2x4, DVec2x4};
use wide::{CmpEq, CmpGt};

use crate::MbState;
use crate::Solver;
use crate::Viewbox;

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
pub struct VecUvCell {
    pub(crate) c: C4,
    pub(crate) z: C4,
    pub(crate) i: f64x4,
}

impl VecUvCell {
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
pub struct VecUvState {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) state: Vec<VecUvCell>,
}

impl MbState for VecUvState {
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn i_value(&self, x: usize, y: usize) -> i16 {
        let n = self.width * y + x;
        self.state[n / 4].i.as_array_ref()[n % 4] as i16
    }
}

impl From<Viewbox> for VecUvState {
    fn from(v: Viewbox) -> Self {
        let cs: Vec<num::complex::Complex<f64>> = v.generate_complex_coordinates();
        assert!(cs.len() % 4 == 0, "oops");
        let mut state = Vec::with_capacity(cs.len() / 4);

        for i in (0..cs.len()).step_by(4) {
            let re = [cs[i].re, cs[i + 1].re, cs[i + 2].re, cs[i + 3].re];
            let im = [cs[i].im, cs[i + 1].im, cs[i + 2].im, cs[i + 3].im];
            let c = c4(f64x4::new(re), f64x4::new(im));
            state.push(VecUvCell {
                c,
                z: c,
                i: f64x4::splat(f64::INFINITY),
            })
        }

        Self {
            width: v.width as usize,
            height: v.height as usize,
            state,
        }
    }
}

pub struct VecUvSolver {
    treshold: f64x4,
}

impl Default for VecUvSolver {
    fn default() -> Self {
        Self {
            treshold: f64x4::splat(2.0),
        }
    }
}

impl Solver<VecUvState> for VecUvSolver {
    fn solve(&self, mut state: VecUvState) -> VecUvState {
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

    #[test]
    fn test_complex4() {
        let treshold = f64x4::splat(100.0);
        let VecUvState { mut state, .. } = Viewbox::initial(8, 1).into();
        let mut cell1 = state.pop().unwrap();
        let mut cell2 = state.pop().unwrap();
        println!("{:?} {:?}", cell1.i, cell2.i);

        cell1.iterate(100, f64x4::splat(0.0), treshold);
        cell2.iterate(100, f64x4::splat(0.0), treshold);
        println!("{:?} {:?}", cell1.i, cell2.i);
    }
}
