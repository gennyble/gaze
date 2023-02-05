/*
We need to be able to represent:
- Sensor data
- RGB
- HSV
- Non-gamma
*/
pub trait Colorspace {
	/// Number of elements per pixel
	const COMPONENTS: usize;
}

/// Straight-from-the-camera colours. Almost certainly linear.
pub struct BayerRgb {}

impl Colorspace for BayerRgb {
	const COMPONENTS: usize = 1;
}

/// Linear RGB.
pub struct LinRgb {}

impl Colorspace for LinRgb {
	const COMPONENTS: usize = 3;
}

pub struct XYZ {}

impl Colorspace for XYZ {
	const COMPONENTS: usize = 3;
}

pub struct LinSrgb {}

impl Colorspace for LinSrgb {
	const COMPONENTS: usize = 3;
}

pub struct Srgb {}

impl Colorspace for Srgb {
	const COMPONENTS: usize = 3;
}
