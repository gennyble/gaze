use nalgebra::{Matrix3, Matrix3x1};

use crate::colorspace::{LinSrgb, XYZ};

use super::Image;

impl Image<u16, XYZ> {
	pub fn to_linsrgb(mut self) -> Image<u16, LinSrgb> {
		let wb = self.metadata.whitebalance;
		let cam_reference = (self.metadata.cam_to_xyz * Matrix3x1::new(1.0, 1.0, 1.0)).normalize();
		let srgb_reference = XYZ_TO_SRGB.try_inverse().unwrap() * Matrix3x1::new(1.0, 1.0, 1.0);

		println!("Cam {cam_reference}\nsRGB {srgb_reference}");

		let cam_cones = BRADFORD * cam_reference;
		let srgb_cones = BRADFORD * srgb_reference;

		#[rustfmt::skip]
		let difference_matrix = Matrix3::new(
			srgb_cones[0] / cam_cones[0], 0.0, 0.0,
			0.0, srgb_cones[1] / cam_cones[1], 0.0,
			0.0, 0.0, srgb_cones[2] / cam_cones[2]
		);

		let chromatic_adaptation_matrix = BRADFORD_INV * difference_matrix * BRADFORD;

		println!("{chromatic_adaptation_matrix}");

		let adapted = chromatic_adaptation_matrix * cam_reference;

		for px in self.data.chunks_mut(3) {
			let m = Matrix3x1::new(
				px[0] as f32 / self.metadata.whitelevels[0] as f32,
				px[1] as f32 / self.metadata.whitelevels[1] as f32,
				px[2] as f32 / self.metadata.whitelevels[2] as f32,
			);
			let res = BRUCE_XYZ_SRGB * (chromatic_adaptation_matrix * m);
			px[0] = (res[0] * self.metadata.whitelevels[0] as f32) as u16;
			px[1] = (res[1] * self.metadata.whitelevels[1] as f32) as u16;
			px[2] = (res[2] * self.metadata.whitelevels[2] as f32) as u16;
		}

		self.change_colorspace(None)
	}
}

// Assumes D65 white
#[rustfmt::skip]
pub const XYZ_TO_SRGB: Matrix3<f32> = Matrix3::new(
	 3.2406, -1.5372, -0.4986,
	-0.9689,  1.8752,  0.0415,
	 0.0057, -0.2040,  1.0570
);

#[rustfmt::skip]
pub const BRUCE_XYZ_SRGB: Matrix3<f32> = Matrix3::new(
3.2404542, -1.5371385, -0.4985314,
-0.9692660,  1.8760108,  0.0415560,
 0.0556434, -0.2040259,  1.0572252
);

// This is a method of "chromatic adaption", which is fancy speak for switching
// between reference whites? I didn't know that. This is how you change the
// whitebalance in an image!
// http://www.brucelindbloom.com/index.html?Eqn_ChromAdapt.html
#[rustfmt::skip]
const XYZ_SCALING: Matrix3<f32> = Matrix3::new(
	1.0, 0.0, 0.0,
	0.0, 1.0, 0.0,
	0.0, 0.0, 1.0
);

const BRADFORD: Matrix3<f32> = Matrix3::new(
	0.8951000, 0.2664000, -0.1614000, -0.7502000, 1.7135000, 0.0367000, 0.0389000, -0.0685000,
	1.0296000,
);

const BRADFORD_INV: Matrix3<f32> = Matrix3::new(
	0.9869929, -0.1470543, 0.1599627, 0.4323053, 0.5183603, 0.0492912, -0.0085287, 0.0400428,
	0.9684867,
);
