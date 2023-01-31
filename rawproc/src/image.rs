use std::marker::PhantomData;

use rawloader::CFA;

use crate::{
	colorspace::{BayerRgb, Colorspace, LinRgb},
	RollingRandom,
};

pub struct RawMetadata {
	/// Whitebalance coefficients. Red, green, blue
	pub whitebalance: [f32; 3],
	/// Whitelevel values; the highest per channel value
	pub whitelevels: [u16; 3],
	pub crop: Option<Crop>,
	pub cfa: CFA,
}

#[derive(Copy, Clone, Debug)]
pub struct Crop {
	pub top: usize,
	pub right: usize,
	pub bottom: usize,
	pub left: usize,
}

impl Crop {
	/// The provided array should be in the order of top, right, bottom, left. If all are 0, None is returned.
	pub fn from_css_quad(v: [usize; 4]) -> Option<Self> {
		if v.iter().sum::<usize>() != 0 {
			Some(Self {
				top: v[0],
				right: v[1],
				bottom: v[2],
				left: v[3],
			})
		} else {
			None
		}
	}
}

pub struct Image<T: Copy + Clone, C: Colorspace> {
	pub width: usize,
	pub height: usize,
	pub metadata: RawMetadata,

	pub data: Vec<T>,
	pub(crate) phantom: PhantomData<C>,
}

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

	pub fn debayer(self) -> Image<T, LinRgb> {
		let mut rgb = vec![self.data[0]; self.width * self.height * 3];

		let cfa = self.metadata.cfa.clone();
		let mut rr = RollingRandom::new();

		#[rustfmt::skip]
		let options = [
			(-1, -1), (0, -1), (1, -1),
			(-1, 0),  /*skip*/ (1, 0),
			(-1, 1),  (0, 1),  (1, 1)
		];

		let get = |p: (usize, usize)| -> T { self.data[self.width * p.1 + p.0] };
		let mut set = |x: usize, y: usize, clr: CfaColor, v: T| {
			rgb[(self.width * y + x) * LinRgb::COMPONENTS + clr.rgb_index()] = v;
		};

		//TODO: gen- care about the edges of the image
		// We're staying away from the borders for now so we can handle them special later
		for x in 1..self.width - 1 {
			for y in 1..self.height - 1 {
				let mut options = options.clone().into_iter().map(|(x_off, y_off)| {
					let x = (x as isize + x_off) as usize;
					let y = (y as isize + y_off) as usize;
					(CfaColor::from(cfa.color_at(x, y)), x, y)
				});

				match CfaColor::from(cfa.color_at(x, y)) {
					#[rustfmt::skip]
					CfaColor::Red => {
						set(x, y, CfaColor::Red, get((x, y)));
						set(x, y, CfaColor::Green, get(pick_color(&mut rr, options.clone(), CfaColor::Green)));
						set(x, y, CfaColor::Blue, get(pick_color(&mut rr, options.clone(), CfaColor::Blue)));
					}
					#[rustfmt::skip]
					CfaColor::Blue => {
						set(x, y, CfaColor::Red, get(pick_color(&mut rr, options.clone(), CfaColor::Red)));
						set(x, y, CfaColor::Blue, get((x, y)));
						set(x, y, CfaColor::Green, get(pick_color(&mut rr, options.clone(), CfaColor::Green)));
					}
					#[rustfmt::skip]
					CfaColor::Green => {
						set(x, y, CfaColor::Red, get(pick_color(&mut rr, options.clone(), CfaColor::Red)));
						set(x, y, CfaColor::Blue, get(pick_color(&mut rr, options.clone(), CfaColor::Blue)));
						set(x, y, CfaColor::Green, get((x, y)));
					}
					CfaColor::Emerald => unreachable!(),
				}
			}
		}

		Image {
			width: self.width,
			height: self.height,
			metadata: self.metadata,
			data: rgb,
			phantom: Default::default(),
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
