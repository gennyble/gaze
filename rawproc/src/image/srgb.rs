use crate::{algorithms, colorspace::Srgb};

use super::Image;

impl Image<f32, Srgb> {
	pub fn contrast(&mut self, value: f32) {
		for px in self.data.iter_mut() {
			*px = algorithms::contrast(*px, value);
		}
	}
}
