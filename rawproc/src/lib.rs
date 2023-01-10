pub mod algorithm;
pub mod debayer;
pub mod image;

use image::SensorImage;
use rawloader::RawImageData;

use crate::image::{Metadata, CFA};

pub fn read_file(filename: &str) -> SensorImage<u16> {
	// Raw NEF data
	let decoded = rawloader::decode_file(filename).unwrap();
	let sensor_data = if let RawImageData::Integer(v) = decoded.data {
		v
	} else {
		panic!()
	};

	let raw_size = decoded.width * decoded.height;
	let image_size = sizes.width as usize * sizes.height as usize;

	// TODO: Move to own function, call `extract_meaningful_image` maybe?
	if raw_size != image_size {
		let mut image = Vec::with_capacity(image_size);

		// FIXME: Assumes the extra data is to the right and/or bottom
		for row in sizes.top_margin as usize..sizes.top_margin as usize + sizes.height as usize {
			let lower = row * sizes.raw_width as usize + sizes.left_margin as usize;
			let upper = lower + sizes.width as usize;

			image.extend_from_slice(&sensor_data[lower..upper]);
		}

		/*println!(
			"raw {}x{} - cropped {}x{} - offset {}x{} - eis {} - ais {}",
			sizes.raw_width,
			sizes.raw_height,
			sizes.width,
			sizes.height,
			sizes.left_margin,
			sizes.top_margin,
			image_size,
			image.len()
		);*/

		//FIXME: Assumes CFA:RGGB
		SensorImage {
			data: image,
			meta: Metadata::new(
				sizes.width as u32,
				sizes.height as u32,
				CFA::RGGB,
				decoded.color(),
			),
		}
	} else {
		//FIXME: Assumes CFA:RGGB
		SensorImage {
			data: sensor_data,
			meta: Metadata::new(
				sizes.raw_width as u32,
				sizes.raw_height as u32,
				CFA::RGGB,
				decoded.color(),
			),
		}
	}
}
