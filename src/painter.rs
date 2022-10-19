use std::fmt::Debug;
use std::sync::Arc;

use druid::piet::ImageFormat;
use druid::ImageBuf;
use image::{Rgb, RgbImage};

use crate::solver::MbState;

pub trait Painter<T> {
    fn paint(&self, t: &T) -> RgbImage;
}

pub trait ColorScale: Clone + Debug {
    fn get_color(&self, frac: f64) -> Rgb<u8>;
}

pub trait IValueGetter<T> {
    fn width(&self, t: &T) -> usize;
    fn height(&self, t: &T) -> usize;
    fn i_value(&self, t: &T, x: usize, y: usize) -> i16;
}

pub struct IValuePainter<C>
where
    C: ColorScale,
{
    max_i_value: i16,
    color: C,
}

impl<C> IValuePainter<C>
where
    C: ColorScale,
{
    pub fn new(color: C, max_i_value: i16) -> Self {
        Self { color, max_i_value }
    }
}

impl<T, C> Painter<T> for IValuePainter<C>
where
    C: ColorScale,
    T: MbState,
{
    fn paint(&self, t: &T) -> RgbImage {
        let width: u32 = t.width().try_into().unwrap();
        let height: u32 = t.height().try_into().unwrap();

        let mut img = RgbImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let i_value = t.i_value(x as usize, y as usize);
                let color = if i_value == -1 {
                    Rgb([0, 0, 0])
                } else {
                    let frac = i_value as f64 / self.max_i_value as f64;
                    let frac = f64::clamp(frac, 0.0, 1.0);
                    self.color.get_color(frac)
                };
                img.put_pixel(x, y, color);
            }
        }

        img
    }
}

#[derive(Clone, Debug)]
pub struct Greyscale;

impl ColorScale for Greyscale {
    fn get_color(&self, frac: f64) -> Rgb<u8> {
        let v: u8 = 255 - (frac * 255.0).round() as u8;
        Rgb([v, v, v])
    }
}

#[derive(Clone, Debug)]
pub struct Rainbow;

fn rainbow_color(n: usize) -> [u8; 3] {
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

impl ColorScale for Rainbow {
    fn get_color(&self, frac: f64) -> Rgb<u8> {
        let n = 9.0 * frac;
        let rem = n - (f64::floor(n));
        let n = n as usize;
        let rgb1 = rainbow_color(n);
        let rgb2 = rainbow_color(n + 1);
        let r = mix(rgb1[0], rgb2[0], rem);
        let g = mix(rgb1[1], rgb2[1], rem);
        let b = mix(rgb1[2], rgb2[2], rem);
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
