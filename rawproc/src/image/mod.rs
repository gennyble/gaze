mod bayerrgb;
mod hsv;
mod linrgb;
mod linsrgb;
mod srgb;
mod xyz;

pub use xyz::XYZ_TO_SRGB;

use std::marker::PhantomData;

use nalgebra::Matrix3;
use rawloader::CFA;

use crate::colorspace::{Colorspace, Hsv, LinSrgb, Srgb};

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

impl<T: Copy + Clone, C: Colorspace> Image<T, C>
where
	Image<T, C>: Into<Image<f32, C>>,
{
	pub fn floats(self) -> Image<f32, C> {
		self.into()
	}
}

impl<T: Copy + Clone, C: Colorspace> Image<T, C>
where
	Image<T, C>: Into<Image<u8, C>>,
{
	pub fn bytes(self) -> Image<u8, C> {
		self.into()
	}
}

impl<T: Copy + Clone, C: Colorspace> Image<T, C>
where
	Image<T, C>: Into<Image<u16, C>>,
{
	pub fn sxiteen(self) -> Image<u16, C> {
		self.into()
	}
}

macro_rules! impl_u16_to_f32 {
	($colorspace:path) => {
		impl From<Image<u16, $colorspace>> for Image<f32, $colorspace> {
			fn from(img: Image<u16, $colorspace>) -> Self {
				let Image {
					width,
					height,
					metadata,
					data: data_u16,
					phantom: _phantom,
				} = img;
				let levels = metadata.whitelevels;

				let data = data_u16
					.into_iter()
					.enumerate()
					.map(|(idx, sixteen)| {
						let color_index = idx % 3;
						sixteen as f32 / levels[color_index] as f32
					})
					.collect();

				Image {
					width,
					height,
					metadata,
					data,
					phantom: Default::default(),
				}
			}
		}
	};
}

impl_u16_to_f32!(Srgb);
impl_u16_to_f32!(LinSrgb);

macro_rules! impl_f32_to_u8 {
	($colorspace:path) => {
		impl From<Image<f32, $colorspace>> for Image<u8, $colorspace> {
			fn from(img: Image<f32, $colorspace>) -> Self {
				let Image {
					width,
					height,
					metadata,
					data: data_u8,
					phantom: _phantom,
				} = img;

				let data = data_u8
					.into_iter()
					.map(|float| (float * 255.0) as u8)
					.collect();

				Image {
					width,
					height,
					metadata,
					data,
					phantom: Default::default(),
				}
			}
		}
	};
}

impl_f32_to_u8!(Srgb);
impl_f32_to_u8!(LinSrgb);
impl_f32_to_u8!(Hsv);

macro_rules! impl_f32_to_u16 {
	($colorspace:path) => {
		impl From<Image<f32, $colorspace>> for Image<u16, $colorspace> {
			fn from(img: Image<f32, $colorspace>) -> Self {
				let Image {
					width,
					height,
					metadata,
					data: data_u8,
					phantom: _phantom,
				} = img;

				let data = data_u8
					.into_iter()
					.map(|float| (float * u16::MAX as f32) as u16)
					.collect();

				Image {
					width,
					height,
					metadata,
					data,
					phantom: Default::default(),
				}
			}
		}
	};
}

impl_f32_to_u16!(Srgb);
impl_f32_to_u16!(LinSrgb);
