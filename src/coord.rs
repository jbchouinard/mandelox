use crate::{complex::*, solver::D2ArrayLike};

#[derive(Copy, Clone, Debug)]
pub struct Point<T>
where
    T: num::Num + Copy,
{
    pub(crate) x: T,
    pub(crate) y: T,
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
    pub fn row_idx(&self, width: T) -> T {
        self.y * width + self.x
    }
    pub fn col_idx(&self, height: T) -> T {
        self.x * height + self.y
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

#[derive(Clone, Debug)]
pub struct Coords<T> {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) values: Vec<T>,
}

impl<T> D2ArrayLike for Coords<T>
where
    T: Default + Clone,
{
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![T::default(); width * height],
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn copy_from(&mut self, other: &Self, from: Point<usize>, to: Point<usize>) {
        let v = other.values[from.row_idx(other.width)].clone();
        self.values[to.row_idx(self.width)] = v;
    }

    fn copy_self(&mut self, from: Point<usize>, to: Point<usize>) {
        let v = self.values[from.row_idx(self.width)].clone();
        self.values[to.row_idx(self.width)] = v;
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

    pub fn generate_complex_coordinates(&self) -> Coords<C<f64>> {
        let mut grid = vec![];
        for (x, y) in self.into_iter() {
            grid.push(self.unscaled(&Point::new(x, y)));
        }
        Coords {
            values: grid,
            width: self.width as usize,
            height: self.height as usize,
        }
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
    use crate::solver::D2ArrayLike;

    fn print_arr(a: &Coords<C<f64>>) {
        for y in 0..a.height() {
            println!("{:?}", &a.values[y * a.width()..(y + 1) * a.width()]);
        }
        println!()
    }

    #[test]
    fn test_2darray() {
        let viewbox = Viewbox::initial(6, 6);
        let mut arr1 = viewbox.generate_complex_coordinates();
        let arr2 = viewbox.generate_complex_coordinates();
        print_arr(&arr1);
        arr1.shift_cols(-6, Some(&arr2.copy_cols(-6)));
        print_arr(&arr1);
    }

    #[test]
    fn test_viewbox_initial() {
        let mut viewbox = Viewbox::initial(6, 8);
        viewbox.center.y = 100;
        viewbox.center.x = 200;
        let xy = viewbox.into_iter();
        assert_eq!(xy.count(), 48);
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
