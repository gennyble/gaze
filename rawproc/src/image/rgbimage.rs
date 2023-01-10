use crate::algorithm::{self, pixel_rgb_to_hsv};

use super::{data_to_u8s, hsvimage::HsvImage, Color, Component, GrayImage, Image, Metadata};

#[derive(Clone, Debug)]
pub struct RgbImage<T: Component> {
	pub data: Vec<T>,
	pub meta: Metadata,
}

impl<T: Component> RgbImage<T> {
	pub(crate) fn component<C: Into<usize>>(&self, x: u32, y: u32, channel: C) -> T {
		self.data[(y as usize * self.meta.width as usize + x as usize) * 3 + channel.into()]
	}

	pub(crate) fn set_component<C: Into<usize>>(&mut self, index: usize, channel: C, value: T) {
		self.data[index * 3 + channel.into()] = value;
	}
}

impl RgbImage<f32> {
	pub fn into_u8s(self) -> RgbImage<u8> {
		RgbImage {
			data: data_to_u8s(&self.meta, self.data),
			meta: self.meta,
		}
	}

	pub fn into_gray(mut self) -> GrayImage<f32> {
		GrayImage {
			data: self
				.data
				.chunks(3)
				.map(|c| (c[0] + c[1] + c[2]) / 3f32)
				.collect(),
			meta: self.meta,
		}
	}

	pub fn into_hsv(mut self) -> HsvImage<f32> {
		for pix in self.pixel_iter_mut() {
			let (h, s, v) = pixel_rgb_to_hsv(pix[0], pix[1], pix[2]);

			pix[0] = h;
			pix[1] = s;
			pix[2] = v;
		}

		HsvImage {
			data: self.data,
			meta: self.meta,
		}
	}

	pub fn to_srgb(&mut self) {
		let mat = self.meta.colordata.rgb_cam;
		for pix in self.pixel_iter_mut() {
			let (r, g, b) = (pix[0], pix[1], pix[2]);

			pix[0] = (mat[0][0] * r + mat[0][1] * g + mat[0][2] * b).clamp(0.0, 1.0);
			pix[1] = (mat[1][0] * r + mat[1][1] * g + mat[1][2] * b).clamp(0.0, 1.0);
			pix[2] = (mat[2][0] * r + mat[2][1] * g + mat[2][2] * b).clamp(0.0, 1.0);
		}

		self.srgb_gamma()
	}

	fn srgb_gamma(&mut self) {
		for component in self.data.iter_mut() {
			// Value taken from Wikipedia page on sRGB
			// https://en.wikipedia.org/wiki/SRGB
			if *component <= 0.0031308 {
				*component = (*component * 12.92).clamp(0.0, 1.0);
			} else {
				*component = (1.055 * component.powf(1.0 / 2.4) - 0.055).clamp(0.0, 1.0);
			}
		}
	}

	// https://math.stackexchange.com/a/906280
	pub fn contrast(&mut self, value: f32) {
		for comp in self.data.iter_mut() {
			algorithm::contrast(comp, value)
		}
	}
}

impl<T: Component> Image<T> for RgbImage<T> {
	type Channel = Color;

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
