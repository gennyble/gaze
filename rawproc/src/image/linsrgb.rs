use crate::colorspace::{LinSrgb, Srgb};

use super::Image;

impl Image<u16, LinSrgb> {
	pub fn gamma(mut self) -> Image<u16, Srgb> {
		for px in self.data.iter_mut() {
			//TOOD: use correct whitelevel
			let mut float = *px as f32 / self.metadata.whitelevels[0] as f32;
			if float <= 0.0031308 {
				float *= 12.92;
			} else {
				float = float.powf(1.0 / 2.4) * 1.055 - 0.055;
			}
			*px = (float.max(0.0).min(1.0) * self.metadata.whitelevels[0] as f32) as u16;
		}

		self.change_colorspace(None)
	}
}
