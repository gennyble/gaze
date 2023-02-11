use std::ops::Range;

use rawloader::CFA;

use crate::{
	colorspace::{BayerRgb, Colorspace, LinRgb},
	RollingRandom,
};

use super::Image;

impl<T: Copy + Clone> Image<T, BayerRgb> {
	/// Crops the raw image down, removing parts we're supposed to.
	///
	/// A camera may cover part of a sensor to gather black level information
	/// or noise information, and this function removes those parts so we can
	/// get just the image itself
	pub fn crop(&mut self) {
		let crop = if let Some(crop) = self.metadata.crop.as_ref() {
			*crop
		} else {
			return;
		};

		let new_width = self.width - (crop.left + crop.right);
		let new_height = self.height - (crop.top + crop.bottom);
		let new_size = new_width * new_height;

		//TODO: gen- we do not need to allocate again here. We, in theory, can
		// do this all in the already existing vec
		let mut image = Vec::with_capacity(new_size);
		for row in 0..new_height {
			let row_x = row + crop.top;

			let start = row_x * self.width + crop.left;
			let end = start + new_width;
			image.extend_from_slice(&self.data[start..end]);
		}

		self.width = new_width;
		self.height = new_height;
		self.data = image;
		self.metadata.crop = None;
		self.metadata.cfa = self.metadata.cfa.shift(crop.left, crop.top);
	}

	fn color_at_i(&self, i: usize) -> CfaColor {
		CfaColor::from(self.metadata.cfa.color_at(i % self.width, i / self.width))
	}

	pub fn debayer(self) -> Image<T, LinRgb> {
		let mut rgb = vec![self.data[0]; self.width * self.height * 3];

		let cfa = self.metadata.cfa.clone();
		let mut rr = RollingRandom::new();

		#[rustfmt::skip]
		let top_options = [
			(-1, 0),  /*skip*/ (1, 0),
			(-1, 1),  (0, 1),  (1, 1)
		];

		#[rustfmt::skip]
		let bottom_options = [
			(-1, -1), (0, -1), (1, -1),
			(-1, 0),  /*skip*/ (1, 0),
		];

		#[rustfmt::skip]
		let left_options = [
			(0, -1), (1, -1),
			/*skip*/ (1, 0),
			(0, 1),  (1, 1)
		];

		#[rustfmt::skip]
		let right_options = [
			(-1, -1), (0, -1),
			(-1, 0),  /*skip*/
			(-1, 1),  (0, 1),
		];

		#[rustfmt::skip]
		let center_options = [
			(-1, -1), (0, -1), (1, -1),
			(-1, 0),  /*skip*/ (1, 0),
			(-1, 1),  (0, 1),  (1, 1)
		];

		let topleft_options = [(1, 0), (0, 1), (1, 1)];
		let topright_options = [(-1, 0), (-1, 1), (0, 1)];
		let bottomleft_options = [(0, -1), (1, -1), (1, 0)];
		let bottomright_options = [(-1, -1), (0, -1), (-1, 0)];

		// These used to be closures but rustc was mad about two mutable refs on rgb
		macro_rules! row {
			($range:expr, $opt:expr) => {
				Self::debayer_meat(
					self.width,
					&mut rgb,
					&cfa,
					&mut rr,
					self.data.as_slice(),
					$range,
					$opt,
				);
			};
		}

		macro_rules! pixel {
			($idx:expr, $opt:expr) => {
				Self::debayer_inner(
					self.width,
					&mut rgb,
					&cfa,
					&mut rr,
					self.data.as_slice(),
					$idx,
					$opt,
				)
			};
		}

		//TODO: gen- care about the edges of the image
		// We're staying away from the borders for now so we can handle them special later
		for y in 1..self.height - 1 {
			let range_start = self.width * y + 1;
			let range_end = self.width * y + (self.width - 1);
			let range = range_start..range_end;
			row!(range, &center_options);
		}

		// Top
		row!(1..self.width - 1, &top_options);

		// Bottom
		row!(
			(self.width * (self.height - 1)) + 1..(self.width * self.height) - 1,
			&bottom_options
		);

		for y in 1..self.height - 1 {
			//left
			pixel!(self.width * y, &left_options);

			//Right
			pixel!(self.width * (y + 1) - 1, &right_options);
		}

		pixel!(0, &topleft_options);
		pixel!(self.width - 1, &topright_options);
		pixel!(self.width * (self.height - 1), &bottomleft_options);
		pixel!(self.width * self.height - 1, &bottomright_options);

		Image {
			width: self.width,
			height: self.height,
			metadata: self.metadata,
			data: rgb,
			phantom: Default::default(),
		}
	}

	/// This is a poorly named function.
	#[inline]
	fn debayer_meat(
		width: usize,
		rgb: &mut Vec<T>,
		cfa: &CFA,
		rr: &mut RollingRandom,
		bayer: &[T],
		range: Range<usize>,
		options: &[(isize, isize)],
	) {
		for idx in range {
			Self::debayer_inner(width, rgb, cfa, rr, bayer, idx, options)
		}
	}

	#[inline]
	fn debayer_inner(
		width: usize,
		rgb: &mut Vec<T>,
		cfa: &CFA,
		rr: &mut RollingRandom,
		bayer: &[T],
		idx: usize,
		options: &[(isize, isize)],
	) {
		let get = |p: (usize, usize)| -> T { bayer[width * p.1 + p.0] };
		let mut set = |x: usize, y: usize, clr: CfaColor, v: T| {
			rgb[(width * y + x) * LinRgb::COMPONENTS + clr.rgb_index()] = v;
		};

		let y = idx / width;
		let x = idx % width;

		let options = options.clone().into_iter().map(|(x_off, y_off)| {
			let x = (x as isize + x_off) as usize;
			let y = (y as isize + y_off) as usize;
			(CfaColor::from(cfa.color_at(x, y)), x, y)
		});

		match CfaColor::from(cfa.color_at(x, y)) {
			#[rustfmt::skip]
				CfaColor::Red => {
					set(x, y, CfaColor::Red, get((x, y)));
					set(x, y, CfaColor::Green, get(pick_color(rr, options.clone(), CfaColor::Green)));
					set(x, y, CfaColor::Blue, get(pick_color(rr, options.clone(), CfaColor::Blue)));
				}
			#[rustfmt::skip]
				CfaColor::Blue => {
					set(x, y, CfaColor::Red, get(pick_color(rr, options.clone(), CfaColor::Red)));
					set(x, y, CfaColor::Blue, get((x, y)));
					set(x, y, CfaColor::Green, get(pick_color(rr, options.clone(), CfaColor::Green)));
				}
			#[rustfmt::skip]
				CfaColor::Green => {
					set(x, y, CfaColor::Red, get(pick_color(rr, options.clone(), CfaColor::Red)));
					set(x, y, CfaColor::Blue, get(pick_color(rr, options.clone(), CfaColor::Blue)));
					set(x, y, CfaColor::Green, get((x, y)));
				}
			CfaColor::Emerald => unreachable!(),
		}
	}
}

impl Image<f32, BayerRgb> {
	pub fn whitebalance(&mut self) {
		let wb = self.metadata.whitebalance;
		for (i, light) in self.data.iter_mut().enumerate() {
			match CfaColor::from(self.metadata.cfa.color_at(i % self.width, i / self.width)) {
				CfaColor::Red => *light = *light as f32 * wb[0],
				CfaColor::Green => *light = *light as f32 * wb[1],
				CfaColor::Blue => *light = *light as f32 * wb[2],
				CfaColor::Emerald => unreachable!(),
			}
		}
	}
}

impl Image<u16, BayerRgb> {
	pub fn whitebalance(&mut self) {
		let wb = self.metadata.whitebalance;
		for (i, light) in self.data.iter_mut().enumerate() {
			/*match CfaColor::from(self.metadata.cfa.color_at(i % self.width, i / self.width)) {
				CfaColor::Red => *light = (*light as f32 * wb[0]) as u16,
				CfaColor::Green => *light = (*light as f32 * wb[1]) as u16,
				CfaColor::Blue => *light = (*light as f32 * wb[2]) as u16,
				CfaColor::Emerald => unreachable!(),
			}*/
			*light = (*light as f32
				* wb[self.metadata.cfa.color_at(i % self.width, i / self.width)]) as u16;
		}
	}
}

impl Image<u8, BayerRgb> {
	pub fn whitebalance(&mut self) {
		let wb = self.metadata.whitebalance;
		for (i, light) in self.data.iter_mut().enumerate() {
			match CfaColor::from(self.metadata.cfa.color_at(i % self.width, i / self.width)) {
				CfaColor::Red => *light = (*light as f32 * wb[0]) as u8,
				CfaColor::Green => *light = (*light as f32 * wb[1]) as u8,
				CfaColor::Blue => *light = (*light as f32 * wb[2]) as u8,
				CfaColor::Emerald => unreachable!(),
			}
		}
	}
}

#[inline]
fn pick_color<I>(roll: &mut RollingRandom, options: I, color: CfaColor) -> (usize, usize)
where
	I: Iterator<Item = (CfaColor, usize, usize)>,
{
	let colors: Vec<(CfaColor, usize, usize)> =
		options.filter(|(clr, _, _)| *clr == color).collect();
	let random = roll.random_u8() % colors.len() as u8;
	let red = &colors[random as usize];

	(red.1, red.2)
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum CfaColor {
	Red,
	Green,
	Blue,
	Emerald,
}

impl CfaColor {
	pub fn rgb_index(&self) -> usize {
		match self {
			CfaColor::Red => 0,
			CfaColor::Green => 1,
			CfaColor::Blue => 2,
			CfaColor::Emerald => unreachable!(),
		}
	}
}

impl From<usize> for CfaColor {
	fn from(value: usize) -> Self {
		match value {
			0 => CfaColor::Red,
			1 => CfaColor::Green,
			2 => CfaColor::Blue,
			3 => CfaColor::Emerald,
			_ => unreachable!(),
		}
	}
}
