#[inline]
pub fn srgb_gamma(mut float: f32) -> f32 {
	if float <= 0.0031308 {
		float *= 12.92;
	} else {
		float = float.powf(1.0 / 2.4) * 1.055 - 0.055;
	}

	float.max(0.0).min(1.0)
}

#[inline]
pub fn contrast(float: f32, adjustment: f32) -> f32 {
	(adjustment * (float - 0.5) + 0.5).clamp(0.0, 1.0)
}
