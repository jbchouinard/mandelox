use num::{traits::NumOps, One};

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

impl Viewbox {
    pub fn new(x: i64, y: i64, w: i64, h: i64, scale: f64) -> Self {
        Self {
            center: Point::new(x, y),
            width: w,
            height: h,
            scale,
        }
    }

    pub fn initial(width: i64, height: i64) -> Self {
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

    pub fn zoom(&mut self, factor: f64) {
        let C { re, im } = self.unscaled(&self.center);
        self.scale *= factor;
        self.center = Point::new(self.scale(re), self.scale(im))
    }

    pub fn scale(&self, coord: f64) -> i64 {
        f64::round(self.scale * coord) as i64
    }

    pub fn unscaled(&self, p: &Point<i64>) -> C<f64> {
        let cx = cr((p.x as f64) / self.scale);
        let cy = ci((p.y as f64) / self.scale);
        cx + cy
    }

    pub fn generate_complex_coordinates(&self) -> Vec<C<f64>> {
        let mut grid = vec![];
        for (x, y) in self.into_iter() {
            grid.push(self.unscaled(&Point::new(x, y)));
        }
        grid
    }
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
        let from_x = self.center.x - (self.width / 2);
        let to_x = from_x + self.width - 1;
        let from_y = self.center.y - (self.height / 2);
        let to_y = from_y + self.height - 1;
        ViewboxIter {
            x: from_x,
            from_x,
            to_x,
            y: from_y,
            to_y,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_viewbox_initial() {
        let mut viewbox = Viewbox::initial(6, 8);
        viewbox.center.y = 100;
        viewbox.center.x = 200;
        let xy: Vec<(i64, i64)> = viewbox.into_iter().collect();
        assert_eq!(xy.len(), 48);
        /*
        for n in 0..6 {
            println!("{:?}", &xy[6 * n..6 * (n + 1)]);
        }
        println!();
        let cxy = viewbox.generate_complex_coordinates();
        for n in 0..6 {
            println!("{:?}", &cxy[6 * n..6 * (n + 1)]);
        }
        println!(
            "\n{} {:?}\n",
            viewbox.unscaled(&viewbox.center),
            &viewbox.center
        );
        viewbox.zoom(1.2);
        let cxy = viewbox.generate_complex_coordinates();
        for n in 0..6 {
            println!("{:?}", &cxy[6 * n..6 * (n + 1)]);
        }
        println!(
            "\n{} {:?}\n",
            viewbox.unscaled(&viewbox.center),
            &viewbox.center
        );
        */
    }
}
