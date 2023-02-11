use crate::{
	algorithms,
	colorspace::{LinSrgb, Srgb},
};

use super::Image;

// ascii art u16
//
//	UUU   UUU  111  6666666
//	UUU   UUU  111  666
//	UUU   UUU  111  66666666
//	UUUUUUUUU  111  666  666
//	 UUUUUUU   111  66666666
//

impl Image<u16, LinSrgb> {
	pub fn gamma(mut self) -> Image<u16, Srgb> {
		for px in self.data.iter_mut() {
			//TOOD: use correct whitelevel
			let mut float = *px as f32 / self.metadata.whitelevels[0] as f32;
			float = algorithms::srgb_gamma(float);
			*px = (float.max(0.0).min(1.0) * self.metadata.whitelevels[0] as f32) as u16;
		}

		self.change_colorspace(None)
	}
}

// ascii art f32
//
//	FFFFFF  333333  222222
//	FF          33      22
//	FFFFF   333333  222222
//	FF          33  22
//	FF      333333  222222
//

impl Image<f32, LinSrgb> {
	pub fn gamma(mut self) -> Image<f32, Srgb> {
		for float in self.data.iter_mut() {
			*float = algorithms::srgb_gamma(*float);
		}

		self.change_colorspace(None)
	}

	pub fn contrast(&mut self, value: f32) {
		for px in self.data.iter_mut() {
			*px = algorithms::contrast(*px, value);
		}
	}
}
