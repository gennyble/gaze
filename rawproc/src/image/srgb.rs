use crate::{algorithms, colorspace::Srgb};

use super::Image;

impl Image<f32, Srgb> {
	pub fn contrast(&mut self, value: f32) {
		for px in self.data.iter_mut() {
			*px = algorithms::contrast(*px, value);
		}
	}

	//TODO: gen- What do we name this, really?
	/// Stretches the image so that the largest of any component is 1.0
	pub fn autolevel(&mut self) {
		let mut large = 0.0;
		for px in self.data.iter() {
			if *px > large {
				large = *px;
			}
		}

		println!("Largest {large}");

		let mult = 1.0 / large;
		self.data.iter_mut().for_each(|f| *f = *f * mult)
	}
}
