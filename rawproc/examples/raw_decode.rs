use std::path::PathBuf;

use jpeg_encoder::Encoder;
use rawproc::debayer::{Debayer, Interpolation};

const INPUT: &'static str = "tests/raw/i_see_you_goose.nef";
const OUTPUT: &'static str = "tests/output/i_see_you_goose.jpg";

fn main() {
	let mut outpath = PathBuf::from(OUTPUT);
	outpath.pop();

	std::fs::create_dir_all(&outpath).unwrap();

	// Decode the raw file into the raw sensor data
	let mut sensor = rawproc::read_file(INPUT);

	// Subtract the black using the values present in the raw file, if any
	sensor.black_levels(None);

	// Convert the image into f32 instead of u16. This is the format used for
	// most of the algorithms.
	let mut sensor_floats = sensor.into_floats();

	// Correct the white balance using the values the photo was shot at.
	sensor_floats.white_balance(None);

	// Debayer the image using Bilinear iterpolation
	let debayer = Debayer::new(sensor_floats);
	let mut debayered_floats = debayer.interpolate(Interpolation::NearestNeighbor);
	debayered_floats.to_srgb();

	let rgb = debayered_floats.into_u8s();

	let encoder = Encoder::new_file(OUTPUT, 80).unwrap();
	encoder
		.encode(
			&rgb.data,
			rgb.meta.width as u16,
			rgb.meta.height as u16,
			jpeg_encoder::ColorType::Rgb,
		)
		.unwrap()
}
