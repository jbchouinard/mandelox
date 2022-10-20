use druid::{Data, Lens};
use num::{traits::NumOps, Num, One};

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

#[derive(Clone, Debug, Data, Lens)]
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

#[derive(Clone, Debug, Data, Lens)]
pub struct Viewport<T> {
    pub x: Axis<T>,
    pub y: Axis<T>,
}

impl<T> Viewport<T>
where
    T: Num + Copy,
{
    pub fn new(x: Axis<T>, y: Axis<T>) -> Self {
        Self { x, y }
    }

    pub fn from_floats(x1: T, x2: T, y1: T, y2: T) -> Self {
        Self::new(Axis::new(x1, x2), Axis::new(y1, y2))
    }

    pub fn from_box(center_x: T, center_y: T, width: T, height: T) -> Self {
        let x1 = center_x - (width / T::two());
        let x2 = center_x + (width / T::two());
        let y1 = center_y - (height / T::two());
        let y2 = center_y + (height / T::two());
        Self::from_floats(x1, x2, y1, y2)
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

impl Viewport<f64> {
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

impl Default for Viewport<f64> {
    fn default() -> Self {
        Self::new(Axis::new(-2.0, 1.0), Axis::new(-1.0, 1.0))
    }
}
