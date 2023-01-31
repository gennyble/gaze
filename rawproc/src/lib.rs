mod colorspace;
mod image;

use std::io::Read;

use colorspace::BayerRgb;
use image::{Image, RawMetadata};
use rand::{thread_rng, Rng};
use rawloader::{RawImageData, RawLoaderError};

use crate::image::Crop;

pub fn decode<R: Read>(reader: &mut R) -> Result<Image<u16, BayerRgb>, Error> {
	let image = rawloader::decode(reader)?;

	// the whitebalance and a few other values are apparently RGBE, which is RGB
	// with a shared exponent. It's weird and I don't entirely understand how to
	// un E then RGB, but I don't have to? wb_coeffs is hardcoded in rawloader to
	// have the E bit as NaN. whitelevels, which is supposed to be RGBE, too, has
	// a constant value of 3880 on every component for me? Even E, which is maybe
	// a little weird.
	//GEN
	// It's not RGBE with E for exponont. RGBE with E for Emerald! Blame sony. Also:
	// https://en.wikipedia.org/wiki/CYGM_filter
	// http://camera-wiki.org/wiki/Canon_PowerShot_Pro70
	// https://www.snappiness.space/testing-the-only-rgbe-sensor-ever-made/
	let wb_coeffs = image.wb_coeffs;
	let whitebalance = [wb_coeffs[0], wb_coeffs[1], wb_coeffs[2]];
	let wl = image.whitelevels;
	let whitelevels = [wl[0], wl[1], wl[2]];
	let crop = Crop::from_css_quad(image.crops);

	let metadata = RawMetadata {
		whitebalance,
		crop,
		whitelevels,
		cfa: image.cfa,
	};

	let data = match image.data {
		RawImageData::Float(_) => return Err(Error::FloatImageData),
		RawImageData::Integer(intu16) => intu16,
	};

	Ok(Image {
		width: image.width,
		height: image.height,
		metadata,
		phantom: Default::default(),

		data,
	})
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("{source}")]
	RawLoaderError {
		#[from]
		source: RawLoaderError,
	},
	#[error("Raw image data was floats. Please talk to gennyble if you want this supported")]
	FloatImageData,
}

struct RollingRandom {
	values: [u8; Self::BUCKET_SIZE],
	index: u16,
}

impl RollingRandom {
	const BUCKET_SIZE: usize = 1024;

	pub fn new() -> Self {
		let mut values = [0u8; Self::BUCKET_SIZE];
		thread_rng().fill(&mut values[..]);

		Self { values, index: 0 }
	}

	pub fn random_bool(&mut self) -> bool {
		self.random_u8() % 2 == 0
	}

	pub fn random_u8(&mut self) -> u8 {
		let value = self.values[self.index as usize];

		self.index += 1;
		if self.index as usize >= Self::BUCKET_SIZE {
			self.index = 0;
		}

		value
	}
}
