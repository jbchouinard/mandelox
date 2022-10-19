use std::sync::Arc;

use druid::Data;
use ndarray::{concatenate, s, Array, Array1, Array2, Axis, Zip};

use crate::complex::*;
use crate::coord::Viewport;
use crate::solver::{MbState, Solver};
use crate::threads::{Join, RangeSplitter, Split};

fn generate_complex_grid(width: usize, height: usize, grid: &Viewport) -> Array2<C<f64>> {
    let x_coords: Array2<C<f64>> = (0..width)
        .map(|n| cr(n as f64))
        .collect::<Array1<C<f64>>>()
        .into_shape((1, width))
        .unwrap();

    let x_b = cr(grid.x.min);
    let x_m = cr(grid.x.length() / (width as f64 - 1.0));

    let x_coords = x_coords * x_m + x_b;

    let y_coords: Array2<C<f64>> = (0..height)
        .map(|n| cr(n as f64))
        .collect::<Array1<C<f64>>>()
        .into_shape((height, 1))
        .unwrap();

    let y_b = cr(grid.y.min);
    let y_m = cr(grid.y.length() / (height as f64 - 1.0));

    let y_coords = (y_coords * y_m + y_b) * ci(1.0);

    &x_coords + &y_coords
}

#[derive(Clone, Debug)]
pub struct MbArrayState {
    width: usize,
    height: usize,
    iteration: i16,
    ca: Arc<Array2<C<f64>>>,
    za: Arc<Array2<C<f64>>>,
    ia: Arc<Array2<i16>>,
}

impl Data for MbArrayState {
    fn same(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.iteration == other.iteration
            && self.ca[[1, 1]] == other.ca[[1, 1]]
    }
}

impl MbState for MbArrayState {
    fn initialize(width: usize, height: usize, scale: &Viewport) -> Self {
        let ca = generate_complex_grid(width, height, scale);
        let za = ca.clone();
        let ia: Array2<i16> = Array::from_elem((height, width), -1);
        Self {
            width,
            height,
            iteration: 0,
            ca: Arc::new(ca),
            za: Arc::new(za),
            ia: Arc::new(ia),
        }
    }
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn i_value(&self, x: usize, y: usize) -> i16 {
        self.ia[[y, x]]
    }
}

impl Split for MbArrayState {
    fn split_to_vec(self, n: usize) -> Vec<Self> {
        let mut split: Vec<Self> = vec![];
        for (m, n) in RangeSplitter::split(0, self.height, n) {
            let slice = s![m..n, ..];
            let ca: Array2<C<f64>> = self.ca.slice(slice).into_owned();
            let za: Array2<C<f64>> = self.za.slice(slice).into_owned();
            let ia: Array2<i16> = self.ia.slice(slice).into_owned();
            split.push(MbArrayState {
                width: self.width,
                height: n - m,
                iteration: self.iteration,
                ca: Arc::new(ca),
                za: Arc::new(za),
                ia: Arc::new(ia),
            })
        }
        split
    }
}

impl Join for MbArrayState {
    fn join_vec(states: Vec<MbArrayState>) -> Self {
        let width = states[0].width;
        let iteration = states[0].iteration;
        let mut height = 0;
        let mut cas = vec![];
        let mut zas = vec![];
        let mut ias = vec![];

        for state in &states {
            if width != state.width {
                panic!("different width");
            }
            if iteration != state.iteration {
                panic!("different iteration");
            }
            height += state.height;
            cas.push(state.ca.as_ref().view());
            zas.push(state.za.as_ref().view());
            ias.push(state.ia.as_ref().view());
        }

        let ca = concatenate(Axis(0), &cas).unwrap();
        let za = concatenate(Axis(0), &zas).unwrap();
        let ia = concatenate(Axis(0), &ias).unwrap();
        MbArrayState {
            width,
            height,
            iteration,
            ca: Arc::new(ca),
            za: Arc::new(za),
            ia: Arc::new(ia),
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
    fn solve(&self, mut state: MbArrayState) -> MbArrayState {
        for _ in 0..self.iterations {
            state = self.iterate(&state);
        }
        state
    }
}
