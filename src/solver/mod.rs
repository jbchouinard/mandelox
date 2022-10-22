use std::cmp::Ordering;

use crate::complex::C;
use crate::coord::{Coords, Point};
use crate::threads::{Call, Join, Split, WorkerPool};

pub mod array;
pub mod simdvec;
pub mod vec;

pub use array::{ArraySolver, ArrayState};
pub use simdvec::{SimdVecSolver, SimdVecState};
pub use vec::{VecSolver, VecState};

pub trait Solver<T> {
    fn solve(&self, state: T) -> T;

    fn threaded(self, n: usize) -> WorkerPool<T, T>
    where
        Self: Clone + Send + 'static,
        T: Split + Join + Send + 'static,
    {
        WorkerPool::with(n, || {
            let solver = self.clone();
            move |state| solver.solve(state)
        })
    }
}

impl<T> Solver<T> for WorkerPool<T, T>
where
    Self: Call<T, T>,
    T: MbState + Split + Join,
{
    fn solve(&self, state: T) -> T {
        self.call(state)
    }
}

pub trait MbState: From<Coords<C<f64>>> {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn i_value(&self, x: usize, y: usize) -> i16;
}

pub trait D2ArrayLike: Sized {
    fn new(width: usize, height: usize) -> Self;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn copy_from(&mut self, other: &Self, from: Point<usize>, to: Point<usize>);
    fn copy_self(&mut self, from: Point<usize>, to: Point<usize>);
    fn copy_slice(&self, p1: Point<usize>, p2: Point<usize>) -> Self {
        let xmax = p1.x.max(p2.x);
        assert!(xmax <= self.width());
        let xmin = p1.x.min(p2.x);
        let ymax = p1.y.max(p2.y);
        assert!(ymax <= self.height());
        let ymin = p1.y.min(p2.y);
        let mut new = Self::new(xmax - xmin, ymax - ymin);
        for y in ymin..ymax {
            for x in xmin..xmax {
                new.copy_from(self, Point::new(x, y), Point::new(x - xmin, y - ymin));
            }
        }

        new
    }
    fn idx(&self, row: i64, col: i64) -> (usize, usize, usize, usize) {
        let h: i64 = self.height().try_into().unwrap();
        let w: i64 = self.width().try_into().unwrap();

        let (row_start, row_end) = if row > 0 {
            (0, row)
        } else {
            assert!(h + row >= 0);
            (h + row, h)
        };
        let (col_start, col_end) = if col > 0 {
            (0, col)
        } else {
            assert!(w + col >= 0);
            (w + col, w)
        };
        (
            row_start.try_into().unwrap(),
            row_end.try_into().unwrap(),
            col_start.try_into().unwrap(),
            col_end.try_into().unwrap(),
        )
    }
    fn copy_cols(&self, col: i64) -> Self {
        let (row_start, row_end, col_start, col_end) =
            self.idx(self.height().try_into().unwrap(), col);
        self.copy_slice(
            Point::new(col_start, row_start),
            Point::new(col_end, row_end),
        )
    }
    fn copy_rows(&self, row: i64) -> Self {
        let (row_start, row_end, col_start, col_end) =
            self.idx(row, self.width().try_into().unwrap());
        self.copy_slice(
            Point::new(col_start, row_start),
            Point::new(col_end, row_end),
        )
    }
    fn merge_rows(a: &Self, b: &Self) -> Self {
        let width = a.width();
        assert!(b.width() == width, "different width");
        let a_h = a.height();
        let b_h = b.height();
        let mut new = Self::new(width, a_h + b_h);
        for y in 0..a_h {
            for x in 0..width {
                new.copy_from(a, Point::new(x, y), Point::new(x, y));
            }
        }
        for y in a_h..a_h + b_h {
            for x in 0..width {
                new.copy_from(b, Point::new(x, y - a_h), Point::new(x, y))
            }
        }
        new
    }
    fn merge_cols(a: &Self, b: &Self) -> Self {
        let height = a.height();
        assert!(b.height() == height, "different height");
        let a_w = a.width();
        let b_w = b.width();
        let mut new = Self::new(a_w + b_w, height);
        for y in 0..height {
            for x in 0..a_w {
                new.copy_from(a, Point::new(x, y), Point::new(x, y));
            }
        }
        for y in 0..height {
            for x in a_w..a_w + b_w {
                new.copy_from(b, Point::new(x - a_w, y), Point::new(x, y))
            }
        }
        new
    }
    fn shift_rows(&mut self, row: i64, copy_from: Option<&Self>) {
        if let Some(copysrc) = copy_from {
            assert!(copysrc.height() == row.unsigned_abs() as usize);
            assert!(copysrc.width() == self.width());
        }
        let h = self.height() as i64;
        let (y_iter, cp_y_offset): (Box<dyn Iterator<Item = usize>>, i64) = match row.cmp(&0) {
            Ordering::Greater => (Box::new((0..self.height()).rev()), row),
            Ordering::Less => (Box::new(0..self.height()), -h),
            Ordering::Equal => return,
        };

        for y in y_iter {
            let yi = y as i64;
            let source_y = yi - row;
            for x in 0..self.width() {
                let p = Point::new(x, y);
                let delta_p = Point::new(x, source_y as usize);
                if source_y >= 0 && source_y < h {
                    self.copy_self(delta_p, p);
                } else if let Some(copy_from) = copy_from {
                    let cp_from_p = Point::new(x, (y as i64 - row + cp_y_offset) as usize);
                    self.copy_from(copy_from, cp_from_p, p);
                }
            }
        }
    }
    fn shift_cols(&mut self, col: i64, copy_from: Option<&Self>) {
        if let Some(copysrc) = copy_from {
            assert!(copysrc.height() == self.height());
            assert!(copysrc.width() == col.unsigned_abs() as usize);
        }
        let w = self.width() as i64;
        let (x_iter, cp_x_offset): (Box<dyn Iterator<Item = usize>>, i64) = match col.cmp(&0) {
            Ordering::Greater => (Box::new((0..self.width()).rev()), col),
            Ordering::Less => (Box::new(0..self.width()), -w),
            Ordering::Equal => return,
        };
        for x in x_iter {
            let xi = x as i64;
            let source_x = xi - col;
            for y in 0..self.height() {
                let p = Point::new(x, y);
                let delta_p = Point::new(source_x as usize, y);
                if source_x >= 0 && source_x < w {
                    self.copy_self(delta_p, p);
                } else if let Some(copy_from) = copy_from {
                    let cp_from_p = Point::new((x as i64 - col + cp_x_offset) as usize, y);
                    self.copy_from(copy_from, cp_from_p, p);
                }
            }
        }
    }
}

pub fn default_solver() -> WorkerPool<VecState, VecState> {
    VecSolver::default().threaded(num_cpus::get_physical())
}
