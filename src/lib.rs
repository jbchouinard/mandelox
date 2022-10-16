use std::rc::Rc;

use ndarray::{prelude::*, Zip};

use crate::complex::{ci, cr, C};
use crate::coord::Viewport;

mod complex;
pub mod coord;
pub mod painter;

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

#[derive(Clone)]
pub struct MandelbrotState {
    width: usize,
    height: usize,
    iteration: i16,
    ca: Rc<Array2<C<f64>>>,
    za: Rc<Array2<C<f64>>>,
    ia: Rc<Array2<i16>>,
}

impl MandelbrotState {
    pub fn initialize(width: usize, height: usize, scale: &Viewport) -> Self {
        let ca = generate_complex_grid(width, height, scale);
        let za = ca.clone();
        let ia: Array2<i16> = Array::from_elem((width, height), -1);
        Self {
            width,
            height,
            iteration: 0,
            ca: Rc::new(ca),
            za: Rc::new(za),
            ia: Rc::new(ia),
        }
    }

    pub fn i_values(&self) -> &Array2<i16> {
        &self.ia
    }
}

#[derive(Clone)]
pub struct MandelbrotSolver {
    treshold: f64,
}

impl MandelbrotSolver {
    pub fn new(treshold: f64) -> Self {
        Self { treshold }
    }

    pub fn iterate(&self, state: &MandelbrotState) -> MandelbrotState {
        let mut new_za = Array2::zeros((state.width, state.height));
        let mut new_ia = Array2::zeros((state.width, state.height));

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

        MandelbrotState {
            height: state.height,
            width: state.width,
            iteration: state.iteration + 1,
            ca: state.ca.clone(),
            za: Rc::new(new_za),
            ia: Rc::new(new_ia),
        }
    }

    pub fn iterate_n(&self, state: &MandelbrotState, n: u16) -> MandelbrotState {
        let mut state = state.clone();
        for _ in 0..n {
            state = self.iterate(&state);
        }
        state
    }
}
