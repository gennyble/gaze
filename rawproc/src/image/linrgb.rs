use nalgebra::Matrix3x1;

use crate::colorspace::{LinRgb, XYZ};

use super::Image;

impl Image<u16, LinRgb> {
	pub fn to_xyz(mut self) -> Image<u16, XYZ> {
		for px in self.data.chunks_mut(3) {
			let m = Matrix3x1::new(
				px[0] as f32 / self.metadata.whitelevels[0] as f32,
				px[1] as f32 / self.metadata.whitelevels[1] as f32,
				px[2] as f32 / self.metadata.whitelevels[2] as f32,
			);
			let res = self.metadata.cam_to_xyz * m;
			px[0] = (res[0] * self.metadata.whitelevels[0] as f32) as u16;
			px[1] = (res[1] * self.metadata.whitelevels[1] as f32) as u16;
			px[2] = (res[2] * self.metadata.whitelevels[2] as f32) as u16;
		}

		self.change_colorspace(None)
	}
}
