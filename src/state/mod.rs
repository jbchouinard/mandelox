use crate::complex::{ci, cr, C};
use crate::coord::Viewport;

use ndarray::{concatenate, prelude::*};

use std::sync::Arc;

pub mod solver;

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
pub struct MbState {
    width: usize,
    height: usize,
    iteration: i16,
    ca: Arc<Array2<C<f64>>>,
    za: Arc<Array2<C<f64>>>,
    ia: Arc<Array2<i16>>,
}

impl MbState {
    pub fn initialize(width: usize, height: usize, scale: &Viewport) -> Self {
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

    pub fn split(&self, n: usize) -> Vec<Self> {
        if (self.height % n) != 0 {
            panic!("cannot split evenly");
        }
        let h = self.height / n;

        let mut split: Vec<Self> = vec![];

        for i in 0..n {
            let slice = s![i * h..(i + 1) * h, ..];
            let ca: Array2<C<f64>> = self.ca.slice(slice).into_owned();
            let za: Array2<C<f64>> = self.za.slice(slice).into_owned();
            let ia: Array2<i16> = self.ia.slice(slice).into_owned();
            split.push(MbState {
                width: self.width,
                height: h,
                iteration: self.iteration,
                ca: Arc::new(ca),
                za: Arc::new(za),
                ia: Arc::new(ia),
            })
        }
        split
    }

    pub fn join(&self, states: &[MbState]) -> Self {
        let mut height = self.height;
        let mut cas = vec![self.ca.as_ref().view()];
        let mut zas = vec![self.za.as_ref().view()];
        let mut ias = vec![self.ia.as_ref().view()];

        for other in states {
            if self.width != other.width {
                panic!("different width");
            }
            if self.iteration != other.iteration {
                panic!("different iteration");
            }
            height += other.height;
            cas.push(other.ca.as_ref().view());
            zas.push(other.za.as_ref().view());
            ias.push(other.ia.as_ref().view());
        }

        let ca = concatenate(Axis(0), &cas).unwrap();
        let za = concatenate(Axis(0), &zas).unwrap();
        let ia = concatenate(Axis(0), &ias).unwrap();
        MbState {
            width: self.width,
            height,
            iteration: self.iteration,
            ca: Arc::new(ca),
            za: Arc::new(za),
            ia: Arc::new(ia),
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn i_values(&self) -> &Array2<i16> {
        &self.ia
    }
}
