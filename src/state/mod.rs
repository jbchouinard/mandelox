use std::sync::Arc;

use druid::Data;
use ndarray::{concatenate, prelude::*};

use crate::complex::{ci, cr, C};
use crate::coord::Viewport;
use crate::threads::Split;

pub mod cell;
pub mod shared;
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

pub trait MbState {
    fn initialize(width: usize, height: usize, grid: &Viewport) -> Self;
}

impl MbState for MbVecState {
    fn initialize(width: usize, height: usize, grid: &Viewport) -> Self {
        let x_b = cr(grid.x.min);
        let x_m = cr(grid.x.length() / (width as f64 - 1.0));
        let y_b = cr(grid.y.min);
        let y_m = cr(grid.y.length() / (height as f64 - 1.0));

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
}

impl Split for MbVecState {
    fn split_parts(self, n: usize) -> Vec<Self> {
        let rows = self.state.split_parts(self.height);
        let row_groups = rows.split_parts(n);

        let mut parts = vec![];
        for row_group in row_groups {
            let height = row_group.len();
            let state = Vec::<MbVecCell>::join_parts(row_group);
            parts.push(Self {
                width: self.width,
                height,
                state,
                iteration: self.iteration,
            })
        }
        parts
    }
    fn join_parts(parts: Vec<Self>) -> Self {
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
            state: Vec::join_parts(state_parts),
        }
    }
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
}

impl MbArrayState {
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

impl Split for MbArrayState {
    fn split_parts(self, n: usize) -> Vec<Self> {
        let h = self.height / n;
        let h_xtra = self.height % n;

        let mut split: Vec<Self> = vec![];

        for i in 0..n {
            let height = if i == (n - 1) { h + h_xtra } else { h };
            let start = i * h;
            let end = start + height;
            let slice = s![start..end, ..];
            let ca: Array2<C<f64>> = self.ca.slice(slice).into_owned();
            let za: Array2<C<f64>> = self.za.slice(slice).into_owned();
            let ia: Array2<i16> = self.ia.slice(slice).into_owned();
            split.push(MbArrayState {
                width: self.width,
                height,
                iteration: self.iteration,
                ca: Arc::new(ca),
                za: Arc::new(za),
                ia: Arc::new(ia),
            })
        }
        split
    }

    fn join_parts(states: Vec<MbArrayState>) -> Self {
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
