#[cfg(feature = "gui")]
use druid::{Data, Lens};
use num::{traits::NumOps, Num, One};

use crate::complex::*;

trait Two {
    fn two() -> Self;
}

impl<T> Two for T
where
    T: One + NumOps,
{
    fn two() -> Self {
        T::one() + T::one()
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "gui", derive(Data, Lens))]
pub struct Axis<T> {
    pub min: T,
    pub max: T,
}

impl<T> Axis<T>
where
    T: Num + Copy,
{
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

    pub fn length(&self) -> T {
        self.max - self.min
    }

    pub fn center(&self) -> T {
        (self.max + self.min) / (T::one() + T::one())
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "gui", derive(Data, Lens))]
pub struct Frame<T> {
    pub x: Axis<T>,
    pub y: Axis<T>,
}

impl<T> Frame<T>
where
    T: Num + Copy,
{
    pub fn new(x: Axis<T>, y: Axis<T>) -> Self {
        Self { x, y }
    }

    pub fn from_nums(x1: T, x2: T, y1: T, y2: T) -> Self {
        Self::new(Axis::new(x1, x2), Axis::new(y1, y2))
    }

    pub fn from_box(center_x: T, center_y: T, width: T, height: T) -> Self {
        let x1 = center_x - (width / T::two());
        let x2 = center_x + (width / T::two());
        let y1 = center_y - (height / T::two());
        let y2 = center_y + (height / T::two());
        Self::from_nums(x1, x2, y1, y2)
    }

    pub fn aspect_ratio(&self) -> T {
        self.x.length() / self.y.length()
    }

    pub fn pan(&mut self, x: T, y: T) {
        self.x.min = self.x.min + x;
        self.x.max = self.x.max + x;
        self.y.min = self.y.min + y;
        self.y.max = self.y.max + y;
    }
}

impl Frame<f64> {
    pub fn pan_relative(&mut self, xfrac: f64, yfrac: f64) {
        self.pan(xfrac * self.x.length(), yfrac * self.y.length());
    }

    pub fn zoom(&mut self, factor: f64) {
        let xc = self.x.center();
        let yc = self.y.center();
        self.x.min = xc + (self.x.min - xc) * factor;
        self.x.max = xc + (self.x.max - xc) * factor;
        self.y.min = yc + (self.y.min - yc) * factor;
        self.y.max = yc + (self.y.max - yc) * factor;
    }
}

impl Default for Frame<f64> {
    fn default() -> Self {
        Self::new(Axis::new(-2.0, 1.0), Axis::new(-1.0, 1.0))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Point<T>
where
    T: num::Num + Copy,
{
    x: T,
    y: T,
}

impl<T> Point<T>
where
    T: num::Num + Copy,
{
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
    pub fn add(&self, point: &Self) -> Self {
        Point::new(self.x + point.x, self.y + point.y)
    }
    pub fn mul(&self, scalar: T) -> Self {
        Point::new(self.x * scalar, self.y * scalar)
    }
}

impl<T> Point<T>
where
    T: num::Num + Copy,
    f64: From<T>,
{
    pub fn as_f64(self) -> Point<f64> {
        Point::<f64>::new(self.x.try_into().unwrap(), self.y.try_into().unwrap())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Viewbox {
    pub(crate) center: Point<i64>,
    pub(crate) width: i64,
    pub(crate) height: i64,
    pub(crate) scale: f64,
}

pub struct ViewboxIter {
    y: i64,
    to_y: i64,
    x: i64,
    from_x: i64,
    to_x: i64,
}

impl ViewboxIter {
    fn incr(&mut self) -> (i64, i64) {
        let current = (self.x, self.y);
        if self.y <= self.to_y {
            if self.x < self.to_x {
                self.x += 1;
            } else {
                self.y += 1;
                self.x = self.from_x;
            }
        }
        current
    }
}

impl Iterator for ViewboxIter {
    type Item = (i64, i64);
    fn next(&mut self) -> Option<(i64, i64)> {
        let (x, y) = self.incr();
        if y <= self.to_y {
            Some((x, y))
        } else {
            None
        }
    }
}

impl IntoIterator for Viewbox {
    type Item = (i64, i64);
    type IntoIter = ViewboxIter;
    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        ViewboxIter {
            x: self.center.x - (self.width / 2),
            from_x: self.center.x - (self.width / 2),
            to_x: self.center.x + (self.width / 2) - 1,
            y: self.center.y - (self.height / 2),
            to_y: self.center.y + (self.height / 2) - 1,
        }
    }
}

impl Viewbox {
    pub fn new(x: i64, y: i64, w: usize, h: usize, scale: f64) -> Self {
        Self {
            center: Point::new(x, y),
            width: w.try_into().unwrap(),
            height: h.try_into().unwrap(),
            scale,
        }
    }

    pub fn scale(&self, coord: f64) -> i64 {
        f64::round(self.scale * coord) as i64
    }
    pub fn unscale(&self, n: i64) -> f64 {
        (n as f64) / self.scale
    }
    pub fn initial(width: usize, height: usize) -> Self {
        let aspect_ratio = width as f64 / height as f64;
        let scale = if aspect_ratio > 1.25 {
            height as f64 / 2.4
        } else {
            width as f64 / 3.0
        };
        let mut this = Self::new(0, 0, width, height, scale);
        this.center.x = this.scale(-0.5);
        this
    }
    pub fn generate_complex_coordinates(&self) -> Vec<C<f64>> {
        let mut grid = vec![];
        for (x, y) in self.into_iter() {
            let cx = cr(self.unscale(x));
            let cy = ci(self.unscale(y));
            grid.push(cx + cy);
        }
        grid
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_viewbox_initial() {
        let viewbox = Viewbox::initial(12, 12);
        let xy: Vec<(i64, i64)> = viewbox.into_iter().collect();
        assert_eq!(xy.len(), 144);
    }
}
