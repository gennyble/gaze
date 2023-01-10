use crate::image::{Color, Image, RgbImage, SensorImage};
use rand::{self, thread_rng, Rng};

pub struct Debayer {
	img: RgbImage<f32>,
}

impl Debayer {
	pub fn new(mut rimg: SensorImage<f32>) -> Self {
		let sensor_len = rimg.data.len();
		rimg.data.resize(sensor_len * 3, 0.0);

		for i in (0..sensor_len).rev() {
			match rimg.meta.color_at_index(i) {
				Color::Red => {
					rimg.data[i * 3] = rimg.data[i];
					rimg.data[i * 3 + 1] = 0.0;
					rimg.data[i * 3 + 2] = 0.0;
				}
				Color::Green => {
					rimg.data[i * 3] = 0.0;
					rimg.data[i * 3 + 1] = rimg.data[i];
					rimg.data[i * 3 + 2] = 0.0;
				}
				Color::Blue => {
					rimg.data[i * 3] = 0.0;
					rimg.data[i * 3 + 1] = 0.0;
					rimg.data[i * 3 + 2] = rimg.data[i];
				}
			}
		}

		Self {
			img: RgbImage {
				data: rimg.data,
				meta: rimg.meta,
			},
		}
	}

	pub fn interpolate(mut self, interpolation: Interpolation) -> RgbImage<f32> {
		match interpolation {
			Interpolation::None => (),
			Interpolation::NearestNeighbor => NearestNeighbor::interpolate(&mut self.img),
			Interpolation::Bilinear => Bilinear::interpolate(&mut self.img),
		}

		self.img
	}
}

struct RollingRandom {
	values: [u8; Self::BUCKET_SIZE],
	index: u16,
}

impl RollingRandom {
	const BUCKET_SIZE: usize = 1024;

	pub fn new() -> Self {
		let mut values = [0u8; Self::BUCKET_SIZE];
		thread_rng().fill(&mut values[..]);

		Self { values, index: 0 }
	}

	pub fn random_bool(&mut self) -> bool {
		self.random_u8() % 2 == 0
	}

	pub fn random_u8(&mut self) -> u8 {
		let value = self.values[self.index as usize];

		self.index += 1;
		if self.index as usize >= Self::BUCKET_SIZE {
			self.index = 0;
		}

		value
	}
}

#[derive(Copy, Clone, Debug)]
pub enum Interpolation {
	None,
	NearestNeighbor,
	Bilinear,
}

// FIXME: Currently assumes RGGB bayering
struct NearestNeighbor;
impl NearestNeighbor {
	fn interpolate(cimg: &mut RgbImage<f32>) {
		let mut rand = RollingRandom::new();

		let mut counter: u64 = 0;
		for pix in cimg.pixel_range() {
			match cimg.meta.color_at_index(pix) {
				Color::Red => {
					cimg.set_component(
						pix,
						Color::Green,
						Self::get_component(&mut rand, cimg, Color::Green, pix, &mut counter),
					);
					cimg.set_component(
						pix,
						Color::Blue,
						Self::get_component(&mut rand, cimg, Color::Blue, pix, &mut counter),
					);
				}
				Color::Green => {
					cimg.set_component(
						pix,
						Color::Red,
						Self::get_component(&mut rand, cimg, Color::Red, pix, &mut counter),
					);
					cimg.set_component(
						pix,
						Color::Blue,
						Self::get_component(&mut rand, cimg, Color::Blue, pix, &mut counter),
					);
				}
				Color::Blue => {
					cimg.set_component(
						pix,
						Color::Red,
						Self::get_component(&mut rand, cimg, Color::Red, pix, &mut counter),
					);
					cimg.set_component(
						pix,
						Color::Green,
						Self::get_component(&mut rand, cimg, Color::Green, pix, &mut counter),
					);
				}
			}
		}
	}

	fn get_component(
		rand: &mut RollingRandom,
		cimg: &RgbImage<f32>,
		color: Color,
		i: usize,
		counter: &mut u64,
	) -> f32 {
		*counter += 1;
		let (x, y) = cimg.meta.itoxy(i);

		let top_color = if y == 0 {
			// There is no top pixel
			false
		} else if y == cimg.meta.height - 1 {
			// There is no bottom pixel
			true
		} else {
			// Use a random top/bottom
			rand.random_bool()
		};

		let left_color = if x == 0 {
			// There is no left color
			false
		} else if x == cimg.meta.width - 1 {
			// There is no right color
			true
		} else {
			// Use a random left/right
			rand.random_bool()
		};

		let color_x = if left_color { x - 1 } else { x + 1 };

		let color_y = if top_color { y - 1 } else { y + 1 };

		let current_color = cimg.meta.color_at_xy(x, y);

		match current_color {
			Color::Red => match color {
				Color::Red => cimg.component(x, y, current_color),
				Color::Green => {
					if rand.random_bool() {
						cimg.component(x, color_y, color)
					} else {
						cimg.component(color_x, y, color)
					}
				}
				Color::Blue => cimg.component(color_x, color_y, color),
			},
			Color::Green => {
				let x_even = x % 2 == 0;

				match color {
					Color::Red => {
						if x_even {
							cimg.component(x, color_y, color)
						} else {
							cimg.component(color_x, y, color)
						}
					}
					Color::Green => cimg.component(x, y, current_color),
					Color::Blue => {
						if x_even {
							cimg.component(color_x, y, color)
						} else {
							cimg.component(x, color_y, color)
						}
					}
				}
			}
			Color::Blue => match color {
				Color::Red => cimg.component(color_x, color_y, color),
				Color::Green => {
					if rand::random() {
						cimg.component(x, color_y, color)
					} else {
						cimg.component(color_x, y, color)
					}
				}
				Color::Blue => cimg.component(x, y, current_color),
			},
		}
	}
}

// FIXME: Currently assumes RGGB bayering
struct Bilinear;
impl Bilinear {
	fn interpolate(img: &mut RgbImage<f32>) {
		for pix in img.pixel_range() {
			match img.meta.color_at_index(pix) {
				Color::Red => {
					let (green, blue) = Self::average_for_red(&img, pix);
					img.set_component(pix, Color::Green, green);
					img.set_component(pix, Color::Blue, blue);
				}
				Color::Green => {
					if img.meta.itoxy(pix).1 % 2 == 1 {
						let (red, blue) = Self::average_for_yeven_green(&img, pix);
						img.set_component(pix, Color::Red, red);
						img.set_component(pix, Color::Blue, blue);
					} else {
						let (blue, red) = Self::average_for_yodd_green(&img, pix);
						img.set_component(pix, Color::Red, red);
						img.set_component(pix, Color::Blue, blue);
					}
				}
				Color::Blue => {
					let (red, green) = Self::average_for_blue(&img, pix);
					img.set_component(pix, Color::Red, red);
					img.set_component(pix, Color::Green, green);
				}
			}
		}
	}

	// Returns green, blue
	fn average_for_red(img: &RgbImage<f32>, i: usize) -> (f32, f32) {
		let (x, y) = img.meta.itoxy(i);
		let (top, right, bottom, left) = Self::edges(img.meta.width, img.meta.height, x, y);

		if top {
			if left {
				(
					(
						img.component(x+1, y, Color::Green) + //Right
						img.component(x, y+1, Color::Green)
						//Bottom
					) / 2.0,
					img.component(x + 1, y + 1, Color::Blue), //Bottom-right
				)
			} else if right {
				(
					(
						img.component(x-1, y, Color::Green) + //Left
						img.component(x, y+1, Color::Green)
						//Bottom
					) / 2.0,
					img.component(x - 1, y + 1, Color::Blue), //Bottom-left
				)
			} else {
				(
					(
						img.component(x+1, y, Color::Green) + //Right
						img.component(x, y+1, Color::Green) + //Bottom
						img.component(x-1, y, Color::Green)
						//Left
					) / 3.0,
					(
						img.component(x+1, y+1, Color::Blue) + //Bottom-right
						img.component(x-1, y+1, Color::Blue)
						//Bottom-left
					) / 2.0,
				)
			}
		} else if bottom {
			if left {
				(
					(
						img.component(x, y-1, Color::Green) + //Top
						img.component(x+1, y, Color::Green)
						//Right
					) / 2.0,
					img.component(x + 1, y - 1, Color::Blue), //Top-right
				)
			} else if right {
				(
					(
						img.component(x, y-1, Color::Green) + //Top
						img.component(x-1, y, Color::Green)
						//Left
					) / 2.0,
					img.component(x - 1, y - 1, Color::Blue), //Top-left
				)
			} else {
				(
					(
						img.component(x, y-1, Color::Green) + //Top
						img.component(x+1, y, Color::Green) + //Right
						img.component(x-1, y, Color::Green)
						//Left
					) / 3.0,
					(
						img.component(x+1, y-1, Color::Blue) + //Top-right
						img.component(x-1, y-1, Color::Blue)
						//Top-left
					) / 2.0,
				)
			}
		} else {
			if right {
				(
					(
						img.component(x, y-1, Color::Green) + //Top
					img.component(x, y+1, Color::Green) + //Bottom
					img.component(x-1, y, Color::Green)
						//Left
					) / 3.0,
					(
						img.component(x-1, y+1, Color::Blue) + //Bottom-left
					img.component(x-1, y-1, Color::Blue)
						//Top-left
					) / 2.0,
				)
			} else if left {
				(
					(
						img.component(x, y-1, Color::Green) + //Top
						img.component(x+1, y, Color::Green) + //Right
						img.component(x, y+1, Color::Green)
						//Bottom
					) / 3.0,
					(
						img.component(x+1, y-1, Color::Blue) + //Top-right
						img.component(x+1, y+1, Color::Blue)
						//Bottom-right
					) / 2.0,
				)
			} else {
				(
					(
						img.component(x, y-1, Color::Green) + //Top
						img.component(x+1, y, Color::Green) + //Right
						img.component(x, y+1, Color::Green) + //Bottom
						img.component(x-1, y, Color::Green)
						//Left
					) / 4.0,
					(
						img.component(x+1, y-1, Color::Blue) + //Top-right
						img.component(x+1, y+1, Color::Blue) + //Bottom-right
						img.component(x-1, y+1, Color::Blue) + //Bottom-left
						img.component(x-1, y-1, Color::Blue)
						//Top-left
					) / 4.0,
				)
			}
		}
	}

	// Returns red, blue
	fn average_for_yeven_green(img: &RgbImage<f32>, i: usize) -> (f32, f32) {
		let (x, y) = img.meta.itoxy(i);
		let (top, right, bottom, left) = Self::edges(img.meta.width, img.meta.height, x, y);

		if top {
			if left {
				(
					img.component(x, y + 1, Color::Red),  //Bottom
					img.component(x + 1, y, Color::Blue), //Right
				)
			} else if right {
				(
					img.component(x, y + 1, Color::Red),  //Bottom
					img.component(x - 1, y, Color::Blue), //Left
				)
			} else {
				(
					img.component(x, y + 1, Color::Red), //Bottom
					(
						img.component(x-1, y, Color::Blue) + //Left
						img.component(x+1, y, Color::Blue)
						//Right
					) / 2.0,
				)
			}
		} else if bottom {
			if left {
				(
					img.component(x, y - 1, Color::Red),  //Top
					img.component(x + 1, y, Color::Blue), //Right
				)
			} else if right {
				(
					img.component(x, y - 1, Color::Red),  //Top
					img.component(x - 1, y, Color::Blue), //Left
				)
			} else {
				(
					img.component(x, y - 1, Color::Red), //Top
					(
						img.component(x-1, y, Color::Blue) + //Left
						img.component(x+1, y, Color::Blue)
						//Right
					) / 2.0,
				)
			}
		} else {
			if right {
				(
					(
						img.component(x, y-1, Color::Red) + //Top
						img.component(x, y+1, Color::Red)
						//Bottom
					) / 2.0,
					img.component(x - 1, y, Color::Blue), //Left
				)
			} else if left {
				(
					(
						img.component(x, y-1, Color::Red) + //Top
						img.component(x, y+1, Color::Red)
						//Bottom
					) / 2.0,
					img.component(x + 1, y, Color::Blue), //Right
				)
			} else {
				(
					(
						img.component(x, y-1, Color::Red) + //Top
						img.component(x, y+1, Color::Red)
						//Bottom
					) / 2.0,
					(
						img.component(x-1, y, Color::Blue) + //Left
						img.component(x+1, y, Color::Blue)
						//Right
					) / 2.0,
				)
			}
		}
	}

	// Returns red, blue
	fn average_for_yodd_green(img: &RgbImage<f32>, i: usize) -> (f32, f32) {
		let (x, y) = img.meta.itoxy(i);
		let (top, right, bottom, left) = Self::edges(img.meta.width, img.meta.height, x, y);

		if top {
			if left {
				(
					img.component(x, y + 1, Color::Blue), //Bottom
					img.component(x + 1, y, Color::Red),  //Right
				)
			} else if right {
				(
					img.component(x, y + 1, Color::Blue), //Bottom
					img.component(x - 1, y, Color::Red),  //Left
				)
			} else {
				(
					img.component(x, y + 1, Color::Blue), //Bottom
					(
						img.component(x-1, y, Color::Red) + //Left
						img.component(x+1, y, Color::Red)
						//Right
					) / 2.0,
				)
			}
		} else if bottom {
			if left {
				(
					img.component(x, y - 1, Color::Blue), //Top
					img.component(x + 1, y, Color::Red),  //Right
				)
			} else if right {
				(
					img.component(x, y - 1, Color::Blue), //Top
					img.component(x - 1, y, Color::Red),  //Left
				)
			} else {
				(
					img.component(x, y - 1, Color::Blue), //Top
					(
						img.component(x-1, y, Color::Red) + //Left
						img.component(x+1, y, Color::Red)
						//Right
					) / 2.0,
				)
			}
		} else {
			if right {
				(
					(
						img.component(x, y-1, Color::Blue) + //Top
						img.component(x, y+1, Color::Blue)
						//Bottom
					) / 2.0,
					img.component(x - 1, y, Color::Red), //Left
				)
			} else if left {
				(
					(
						img.component(x, y-1, Color::Blue) + //Top
						img.component(x, y+1, Color::Blue)
						//Bottom
					) / 2.0,
					img.component(x + 1, y, Color::Red), //Right
				)
			} else {
				(
					(
						img.component(x, y-1, Color::Blue) + //Top
						img.component(x, y+1, Color::Blue)
						//Bottom
					) / 2.0,
					(
						img.component(x-1, y, Color::Red) + //Left
						img.component(x+1, y, Color::Red)
						//Right
					) / 2.0,
				)
			}
		}
	}

	// Returns red, green
	fn average_for_blue(img: &RgbImage<f32>, i: usize) -> (f32, f32) {
		let (x, y) = img.meta.itoxy(i);
		let (top, right, bottom, left) = Self::edges(img.meta.width, img.meta.height, x, y);

		if top {
			if left {
				(
					img.component(x + 1, y + 1, Color::Red), //Bottom-right
					(
						img.component(x+1, y, Color::Green) + //Right
						img.component(x, y+1, Color::Green)
						//Bottom
					) / 2.0,
				)
			} else if right {
				(
					img.component(x - 1, y + 1, Color::Red), //Bottom-left
					(
						img.component(x-1, y, Color::Green) + //Left
						img.component(x, y+1, Color::Green)
						//Bottom
					) / 2.0,
				)
			} else {
				(
					(
						img.component(x+1, y+1, Color::Red) + //Bottom-right
						img.component(x-1, y+1, Color::Red)
						//Bottom-left
					) / 2.0,
					(
						img.component(x+1, y, Color::Green) + //Right
						img.component(x, y+1, Color::Green) + //Bottom
						img.component(x-1, y, Color::Green)
						//Left
					) / 3.0,
				)
			}
		} else if bottom {
			if left {
				(
					img.component(x + 1, y - 1, Color::Red), //Top-right
					(
						img.component(x, y-1, Color::Green) + //Top
						img.component(x+1, y, Color::Green)
						//Right
					) / 2.0,
				)
			} else if right {
				(
					img.component(x - 1, y - 1, Color::Red), //Top-left
					(
						img.component(x, y-1, Color::Green) + //Top
						img.component(x-1, y, Color::Green)
						//Left
					) / 2.0,
				)
			} else {
				(
					(
						img.component(x+1, y-1, Color::Red) + //Top-right
						img.component(x-1, y-1, Color::Red)
						//Top-left
					) / 2.0,
					(
						img.component(x, y-1, Color::Green) + //Top
						img.component(x+1, y, Color::Green) + //Right
						img.component(x-1, y, Color::Green)
						//Left
					) / 3.0,
				)
			}
		} else {
			if right {
				(
					(
						img.component(x-1, y+1, Color::Red) + //Bottom-left
					img.component(x-1, y-1, Color::Red)
						//Top-left
					) / 2.0,
					(
						img.component(x, y-1, Color::Green) + //Top
					img.component(x, y+1, Color::Green) + //Bottom
					img.component(x-1, y, Color::Green)
						//Left
					) / 3.0,
				)
			} else if left {
				(
					(
						img.component(x+1, y-1, Color::Red) + //Top-right
						img.component(x+1, y+1, Color::Red)
						//Bottom-right
					) / 2.0,
					(
						img.component(x, y-1, Color::Green) + //Top
						img.component(x+1, y, Color::Green) + //Right
						img.component(x, y+1, Color::Green)
						//Bottom
					) / 3.0,
				)
			} else {
				(
					(
						img.component(x+1, y-1, Color::Red) + //Top-right
						img.component(x+1, y+1, Color::Red) + //Bottom-right
						img.component(x-1, y+1, Color::Red) + //Bottom-left
						img.component(x-1, y-1, Color::Red)
						//Top-left
					) / 4.0,
					(
						img.component(x, y-1, Color::Green) + //Top
						img.component(x+1, y, Color::Green) + //Right
						img.component(x, y+1, Color::Green) + //Bottom
						img.component(x-1, y, Color::Green)
						//Left
					) / 4.0,
				)
			}
		}
	}

	fn edges(w: u32, h: u32, x: u32, y: u32) -> (bool, bool, bool, bool) {
		// Like CSS: Top, Right, Bottom, Left
		(y < 1, x >= w - 1, y >= h - 1, x < 1)
	}
}
