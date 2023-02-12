/*
We need to be able to represent:
- Sensor data
- RGB
- HSV
- Non-gamma
*/
pub trait Colorspace: Clone {
	/// Number of elements per pixel
	const COMPONENTS: usize;
}

/// Straight-from-the-camera colours. Almost certainly linear.
#[derive(Clone, Debug)]
pub struct BayerRgb {}

impl Colorspace for BayerRgb {
	const COMPONENTS: usize = 1;
}

/// Linear RGB.
#[derive(Clone, Debug)]
pub struct LinRgb {}

impl Colorspace for LinRgb {
	const COMPONENTS: usize = 3;
}

#[derive(Clone, Debug)]
pub struct XYZ {}

impl Colorspace for XYZ {
	const COMPONENTS: usize = 3;
}

#[derive(Clone, Debug)]
pub struct LinSrgb {}

impl Colorspace for LinSrgb {
	const COMPONENTS: usize = 3;
}

#[derive(Clone, Debug)]
pub struct Srgb {}

impl Colorspace for Srgb {
	const COMPONENTS: usize = 3;
}

//TODO: gen- Not really a colorspace but more like, representation?
#[derive(Clone, Debug)]
pub struct Hsv {}

impl Colorspace for Hsv {
	const COMPONENTS: usize = 3;
}
