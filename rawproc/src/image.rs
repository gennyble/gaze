use crate::colorspace::{BayerRgb, Colorspace};

pub struct RawMetadata {
	/// Whitebalance coefficients. Red, green, blue
	pub whitebalance: [f32; 3],
	/// Whitelevel values; the highest per channel value
	pub whitelevels: [u16; 3],
	pub crop: Option<Crop>,
}

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

pub struct Image<T: Clone, C: Colorspace> {
	pub width: usize,
	pub height: usize,

	pub data: Vec<T>,
	pub colorspace: C,
}

impl<T: Clone> Image<T, BayerRgb> {
	/// Crops the raw image down, removing parts we're supposed to.
	///
	/// A camera may cover part of a sensor to gather black level information
	/// or noise information, and this function removes those parts so we can
	/// get just the image itself
	pub fn crop(&mut self) {
		let crop = if let Some(crop) = self.colorspace.metadata.crop.as_ref() {
			crop
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
		self.colorspace.metadata.crop = None;
	}
}
