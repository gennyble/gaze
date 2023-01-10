use crate::algorithm::pixel_hsv_to_rgb;

use super::{Attribute, Component, Image, Metadata, RgbImage};

pub struct HsvImage<T: Component> {
    pub data: Vec<T>,
    pub meta: Metadata,
}

impl HsvImage<f32> {
    pub fn into_rgb(mut self) -> RgbImage<f32> {
        for pix in self.pixel_iter_mut() {
            let (r, g, b) = pixel_hsv_to_rgb(pix[0], pix[1], pix[2]);

            pix[0] = r;
            pix[1] = g;
            pix[2] = b;
        }

        RgbImage {
            data: self.data,
            meta: self.meta,
        }
    }

    pub fn saturation(&mut self, scalar: f32) {
        for saturation in self.channel_iter_mut(Attribute::Saturation) {
            *saturation = (*saturation * scalar).clamp(0.0, 1.0);
        }
    }

    pub fn hue_shift(&mut self, shift: f32) {
        for hue in self.channel_iter_mut(Attribute::Hue) {
            *hue = (*hue + shift) % 360.0;
        }
    }

    // https://math.stackexchange.com/a/906280
    pub fn brightness(&mut self, value: f32) {
        for comp in self.channel_iter_mut(Attribute::Value) {
            *comp = (*comp + value).clamp(0.0, 1.0);
        }
    }
}

impl<T: Component> Image<T> for HsvImage<T> {
    type Channel = Attribute;

    fn data(&self) -> &[T] {
        &self.data
    }

    fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    fn meta(&self) -> &Metadata {
        &self.meta
    }

    fn samples(&self) -> usize {
        3
    }
}
