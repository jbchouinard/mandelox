#![allow(clippy::new_without_default)]
use image::RgbImage;

use crate::coord::Viewbox;
use crate::painter::{ColorScale, IValuePainter, Painter};
use crate::solver::{MbState, Solver};
use crate::threads::{Join, Split};

pub mod bench;
mod complex;
pub mod coord;
#[cfg(feature = "gui")]
pub mod gui;
pub mod painter;
pub mod solver;
pub mod threads;

pub struct Mandelbrot<T> {
    pub solver: Box<dyn Solver<T>>,
    pub state: T,
    pub position: Viewbox,
    pub width: usize,
    pub height: usize,
}

impl<T> Mandelbrot<T>
where
    T: MbState + Split + Join + Send + 'static,
{
    pub fn initialize<S>(width: usize, height: usize) -> Self
    where
        S: Solver<T> + Default + Clone + Send + 'static,
    {
        let position = Viewbox::initial(width, height);
        let solver = S::default().threaded(num_cpus::get_physical());
        let initial: T = position.into();
        let solved = solver.solve(initial);
        Self {
            width,
            height,
            position,
            state: solved,
            solver: Box::new(solver),
        }
    }

    pub fn paint<C>(&self, color: C, max_i_value: i16) -> RgbImage
    where
        C: ColorScale,
    {
        let painter = IValuePainter::new(color, max_i_value);
        painter.paint(&self.state)
    }
}
