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

#[inline]
pub fn pixel_rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
	let value = r.max(g.max(b));
	let x_min = r.min(g.min(b));
	let chroma = value - x_min;

	let hue = if chroma == 0.0 {
		0.0
	} else if value == r {
		60.0 * ((g - b) / chroma)
	} else if value == g {
		60.0 * (2.0 + (b - r) / chroma)
	} else if value == b {
		60.0 * (4.0 + (r - g) / chroma)
	} else {
		unreachable!()
	};

	let value_saturation = if value == 0.0 { 0.0 } else { chroma / value };

	/* Rotate the color wheel counter clockwise to remove the negative location
		  |       Keep the wheel in place and remove any full rotations
	 _____V____ _____V____
	|          |          |*/
	((hue + 360.0) % 360.0, value_saturation, value)
}

#[inline]
pub fn pixel_hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (f32, f32, f32) {
	let chroma = value * saturation;
	let hue_prime = hue / 60.0;
	let x = chroma * (1.0 - (hue_prime % 2.0 - 1.0).abs());

	let m = value - chroma;
	let cm = chroma + m;
	let xm = x + m;

	if 0.0 <= hue_prime && hue_prime <= 1.0 {
		(cm, xm, m)
	} else if 1.0 < hue_prime && hue_prime <= 2.0 {
		(xm, cm, m)
	} else if 2.0 < hue_prime && hue_prime <= 3.0 {
		(m, cm, xm)
	} else if 3.0 < hue_prime && hue_prime <= 4.0 {
		(m, xm, cm)
	} else if 4.0 < hue_prime && hue_prime <= 5.0 {
		(xm, m, cm)
	} else if 5.0 < hue_prime && hue_prime <= 6.0 {
		(cm, m, xm)
	} else {
		panic!(
			"This shouldn't be able to happen! HSV ({},{},{})",
			hue, saturation, value
		);
	}
}
