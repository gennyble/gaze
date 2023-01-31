mod colorspace;
mod image;

use std::io::Read;

use colorspace::BayerRgb;
use image::{Image, RawMetadata};
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
	};

	let data = match image.data {
		RawImageData::Float(_) => return Err(Error::FloatImageData),
		RawImageData::Integer(intu16) => intu16,
	};

	Ok(Image {
		width: image.width,
		height: image.height,
		colorspace: BayerRgb { metadata },

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
