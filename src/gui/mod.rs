use std::sync::Arc;

use druid::piet::ImageFormat;
use druid::{Data, ImageBuf};
use image::RgbImage;

use crate::solver::{MbArrayState, MbVecState};

pub mod updater;
pub mod widget;

impl Data for MbArrayState {
    fn same(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.iteration == other.iteration
            && self.ca[[1, 1]] == other.ca[[1, 1]]
    }
}

impl Data for MbVecState {
    fn same(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.iteration == other.iteration
            && self.state[self.width + 2].c == other.state[self.width + 2].c
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
