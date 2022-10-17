use druid::{Data, Lens};

#[derive(Clone, Debug, Data, Lens)]
pub struct Axis {
    pub min: f64,
    pub max: f64,
}

impl Axis {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    pub fn length(&self) -> f64 {
        self.max - self.min
    }

    pub fn center(&self) -> f64 {
        (self.max + self.min) / 2.0
    }
}

#[derive(Clone, Debug, Data, Lens)]
pub struct Viewport {
    pub x: Axis,
    pub y: Axis,
}

impl Viewport {
    pub fn new(x: Axis, y: Axis) -> Self {
        Self { x, y }
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.x.length() / self.y.length()
    }

    pub fn pan(&mut self, x: f64, y: f64) {
        self.x.min += x;
        self.x.max += x;
        self.y.min += y;
        self.y.max += y;
    }

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

impl Default for Viewport {
    fn default() -> Self {
        Self::new(Axis::new(-2.0, 1.0), Axis::new(-1.0, 1.0))
    }
}
