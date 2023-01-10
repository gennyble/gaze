use num_traits::FromPrimitive;

use super::{data_to_floats, Color, Component, Image, IntComponent, Metadata, SingleChannel};

#[derive(Clone, Debug)]
pub struct SensorImage<T: Component> {
	pub data: Vec<T>,
	pub meta: Metadata,
}

impl<T: IntComponent> SensorImage<T> {
	pub fn into_floats(self) -> SensorImage<f32> {
		SensorImage {
			data: data_to_floats(&self.meta, self.data),
			meta: self.meta,
		}
	}
}

impl<T: Component + FromPrimitive> SensorImage<T> {
	// Understanding black levels better:
	// https://chdk.setepontos.com/index.php?topic=7069.0
	pub fn black_levels(&mut self, subtract: Option<(T, T, T)>) {
		let clamp = |light: T, color: T| {
			if light < color {
				T::zero()
			} else {
				light - color
			}
		};

		let (red, green, blue) = match subtract {
			Some(tup) => tup,
			None => {
				let black = self.meta.colordata.black;
				(
					T::from_u32(black).unwrap(),
					T::from_u32(black).unwrap(),
					T::from_u32(black).unwrap(),
				)
			}
		};

		let mut i = 0;
		for light in self.data.iter_mut() {
			match self.meta.color_at_index(i) {
				Color::Red => *light = clamp(*light, red),
				Color::Green => *light = clamp(*light, green),
				Color::Blue => *light = clamp(*light, blue),
			}
			i += 1;
		}
	}
}

impl SensorImage<f32> {
	/// Adjust the exposure in the image. A value of +1 is similar to exposing
	/// one stop higher and -1 a stop lower.
	pub fn exposure(&mut self, ev: f32) {
		for light in self.data.iter_mut() {
			// https://photo.stackexchange.com/a/41936
			// https://github.com/gennyble/rawproc/issues/6#issuecomment-982289991
			*light = (*light * 2f32.powf(ev)).clamp(0.0, 1.0);
		}
	}

	/// Adjust the white balance of the image.
	///
	/// If `balance` is None, the camera's white balance coefficients as read
	/// from the raw file will be used. If `balance` is present, they will be
	/// used as the red, green, and blue coefficients.
	pub fn white_balance(&mut self, balance: Option<(f32, f32, f32)>) {
		let (red, green, blue) = match balance {
			Some(tup) => tup,
			None => {
				let mut whites = self.meta.colordata.cam_mul;

				// Normalize coefficients to green if it's not 1.0
				if whites[1] != 1.0 {
					whites[0] /= whites[1];
					whites[2] /= whites[1];
					whites[1] /= whites[1];
				}

				(whites[0], whites[1], whites[2])
			}
		};

		for (index, light) in self.data.iter_mut().enumerate() {
			match self.meta.color_at_index(index) {
				Color::Red => *light = (*light * red).clamp(0.0, 1.0),
				Color::Green => *light = (*light * green).clamp(0.0, 1.0),
				Color::Blue => *light = (*light * blue).clamp(0.0, 1.0),
			}
		}
	}

	/// Do simple gamma adjustment to the image.
	///
	/// To produce correct looking images you should not use this if you're
	/// going to convert to sRGB as gamma is applied in the sRGB conversion
	/// functions.
	pub fn simple_gamma(&mut self, value: f32) {
		for light in self.data.iter_mut() {
			*light = (*light).powf(1.0 / value).clamp(0.0, 1.0);
		}
	}
}

impl<T: Component> Image<T> for SensorImage<T> {
	type Channel = SingleChannel;

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
		1
	}
}
