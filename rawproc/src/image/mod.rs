mod bayerrgb;
mod linrgb;
mod linsrgb;
mod xyz;

pub use xyz::XYZ_TO_SRGB;

use std::marker::PhantomData;

use nalgebra::Matrix3;
use rawloader::CFA;

use crate::colorspace::Colorspace;

#[derive(Clone, Debug)]
pub struct RawMetadata {
	/// Whitebalance coefficients. Red, green, blue
	pub whitebalance: [f32; 3],
	/// Whitelevel values; the highest per channel value
	pub whitelevels: [u16; 3],
	pub crop: Option<Crop>,
	pub cfa: CFA,
	pub cam_to_xyz: Matrix3<f32>,
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

#[derive(Clone, Debug)]
pub struct Image<T: Copy + Clone, C: Colorspace> {
	pub width: usize,
	pub height: usize,
	pub metadata: RawMetadata,

	pub data: Vec<T>,
	pub(crate) phantom: PhantomData<C>,
}

impl<T: Copy + Clone, C: Colorspace> Image<T, C> {
	pub fn from_raw_parts(
		width: usize,
		height: usize,
		metadata: RawMetadata,
		data: Vec<T>,
	) -> Image<T, C> {
		//TODO: gen- check data is correct length
		Image {
			width,
			height,
			metadata,
			data,
			phantom: Default::default(),
		}
	}

	pub(crate) fn change_colorspace<N: Colorspace>(self, data: Option<Vec<T>>) -> Image<T, N> {
		Image {
			width: self.width,
			height: self.height,
			metadata: self.metadata,
			data: data.unwrap_or(self.data),
			phantom: Default::default(),
		}
	}
}
