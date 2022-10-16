use std::sync::Arc;

use druid::piet::ImageFormat;
use druid::ImageBuf;
use image::{Rgb, RgbImage};
use ndarray::Array2;

pub trait Painter {
    fn i_value_color(&self, i_value: i16) -> Rgb<u8>;

    fn paint(&self, i_values: &Array2<i16>) -> RgbImage {
        let width: u32 = i_values.ncols().try_into().unwrap();
        let height: u32 = i_values.nrows().try_into().unwrap();

        let mut img = RgbImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let i_value = i_values[[y as usize, x as usize]];
                let color = if i_value == -1 {
                    Rgb([0, 0, 0])
                } else {
                    self.i_value_color(i_value)
                };
                img.put_pixel(x, y, color);
            }
        }
        img
    }
}

pub struct GreyscalePainter {
    max_i_value: f64,
}

impl GreyscalePainter {
    pub fn new(max_i_value: f64) -> Self {
        Self { max_i_value }
    }
}
impl Painter for GreyscalePainter {
    fn i_value_color(&self, i_value: i16) -> Rgb<u8> {
        let frac: f64 = i_value as f64 / self.max_i_value;
        let frac = frac.clamp(0.0, 1.0);
        let v: u8 = 255 - (frac * 255.0).round() as u8;
        Rgb([v, v, v])
    }
}

pub struct RainbowPainter {
    max_i_value: f64,
}

impl RainbowPainter {
    pub fn new(max_i_value: f64) -> Self {
        Self { max_i_value }
    }
}

fn rainbow_color(n: i16) -> [u8; 3] {
    match n {
        0 => [0xbe, 0x0a, 0xff],
        1 => [0x58, 0x0a, 0xff],
        2 => [0x14, 0x7d, 0xf5],
        3 => [0x0a, 0xef, 0xff],
        4 => [0x0a, 0xff, 0x99],
        5 => [0xa1, 0xff, 0x0a],
        6 => [0xde, 0xff, 0x0a],
        7 => [0xff, 0xd3, 0x00],
        8 => [0xff, 0x87, 0x00],
        _ => [0xff, 0x00, 0x00],
    }
}

fn mix(a: u8, b: u8, frac: f64) -> u8 {
    let af = a as f64;
    let bf = b as f64;
    let m = af * (1.0 - frac) + bf * frac;
    f64::round(m) as u8
}

impl Painter for RainbowPainter {
    fn i_value_color(&self, i_value: i16) -> Rgb<u8> {
        let n = (9 * i_value) / self.max_i_value as i16;
        let frac = ((9.0 * i_value as f64) / self.max_i_value) - (n as f64);
        let rgb1 = rainbow_color(n);
        let rgb2 = rainbow_color(n + 1);
        let r = mix(rgb1[0], rgb2[0], frac);
        let g = mix(rgb1[1], rgb2[1], frac);
        let b = mix(rgb1[2], rgb2[2], frac);
        Rgb([r, g, b])
    }
}

pub fn convert_image(img: RgbImage) -> ImageBuf {
    let raw: Arc<[u8]> = img.as_raw().clone().into();

    ImageBuf::from_raw(
        raw,
        ImageFormat::Rgb,
        img.width() as usize,
        img.height() as usize,
    )
}
