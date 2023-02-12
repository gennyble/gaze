use crate::{
	algorithms,
	colorspace::{Hsv, Srgb},
};

use super::Image;

impl Image<f32, Hsv> {
	pub fn saturation(&mut self, scalar: f32) {
		for hsv in self.data.chunks_mut(3) {
			hsv[1] = hsv[1] * scalar;
		}
	}
}

impl From<Image<f32, Srgb>> for Image<f32, Hsv> {
	fn from(mut value: Image<f32, Srgb>) -> Self {
		value.data.chunks_mut(3).for_each(|rgb| {
			let (r, g, b) = (rgb[0], rgb[1], rgb[2]);
			let (h, s, v) = algorithms::pixel_rgb_to_hsv(r, g, b);
			rgb[0] = h;
			rgb[1] = s;
			rgb[2] = v;
		});

		value.change_colorspace(None)
	}
}

impl From<Image<f32, Hsv>> for Image<f32, Srgb> {
	fn from(mut value: Image<f32, Hsv>) -> Self {
		value.data.chunks_mut(3).for_each(|hsv| {
			let (h, s, v) = (hsv[0], hsv[1], hsv[2]);
			let (r, g, b) = algorithms::pixel_hsv_to_rgb(h, s, v);
			hsv[0] = r;
			hsv[1] = g;
			hsv[2] = b;
		});

		value.change_colorspace(None)
	}
}
