/*
We need to be able to represent:
- Sensor data
- RGB
- HSV
- Non-gamma
*/

use crate::image::RawMetadata;

pub trait Colorspace {
	/// Number of elements per pixel
	const COMPONENTS: usize;
}

/// Straight-from-the-camera colours. Almost certainly linear.
pub struct BayerRgb {
	pub metadata: RawMetadata,
}

impl Colorspace for BayerRgb {
	const COMPONENTS: usize = 1;
}
