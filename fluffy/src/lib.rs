#[cfg(feature = "winit")]
use winit::dpi::PhysicalSize;

pub struct Buffer {
	/// Bytes - 0RGB
	pub data: Vec<u32>,
	pub width: usize,
	pub height: usize,
}

impl Buffer {
	pub fn new(width: usize, height: usize) -> Self {
		Buffer {
			data: Vec::with_capacity(width * height),
			width,
			height,
		}
	}

	pub fn clear(&mut self) {
		self.data.fill(0)
	}

	#[cfg(feature = "winit")]
	pub fn resize_from_physical(&mut self, phys: PhysicalSize<u32>) {
		self.resize(phys.width as usize, phys.height as usize)
	}

	pub fn resize(&mut self, width: usize, height: usize) {
		self.width = width;
		self.height = height;
		self.data.resize(self.width * self.height, 0);
	}

	/// Yeah. That's what I'm calling this, really.
	///
	/// Takes a (f64, f64) tuple in the range [0,1000] and rescales it to fall
	/// within the width and height so you can get (x,y)
	pub fn dethou(&self, tup: (f64, f64)) -> (usize, usize) {
		(
			(tup.0 * (self.width - 1) as f64 / 1000.0).floor() as usize,
			(tup.1 * (self.height - 1) as f64 / 1000.0).floor() as usize,
		)
	}

	/// Set a pixel with the RGB value
	pub fn set(&mut self, x: usize, y: usize, c: Color) {
		if y >= self.height || x >= self.width {
			return;
		}

		self.set_unchecked(x, y, c)
	}

	pub fn set_unchecked(&mut self, x: usize, y: usize, c: Color) {
		let px = &mut self.data[y * self.width + x];
		*px = c.u32();
	}

	pub fn rect(&mut self, x: usize, y: usize, width: usize, height: usize, c: Color) {
		//TODO: check x and y are in range before we loop so we don't check every time
		for px in x..x + width {
			for py in y..y + height {
				self.set(px, py, c)
			}
		}
	}

	/// Draw a vertical line :D
	/// Range is [y_start,y_end). I.E. start is incldued, end is not. If start
	/// is greater than end, the two are swapped.
	pub fn vert(&mut self, x: usize, y_start: usize, y_end: usize, c: Color) {
		let ymin = y_start.min(y_end);
		let ymax = y_start.max(y_end).clamp(0, self.height);

		for y in ymin..ymax {
			self.set_unchecked(x, y, c);
		}
	}

	/// Draw a horizontal line :D
	/// Range is [x_start,x_end). I.E. start is included, end is not. If start
	/// is greater than end, the two are swapped
	pub fn hori(&mut self, y: usize, x_start: usize, x_end: usize, c: Color) {
		let xmin = x_start.min(x_end);
		let xmax = x_start.max(x_end).clamp(0, self.width);

		for x in xmin..xmax {
			self.set_unchecked(x, y, c);
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

impl Color {
	pub const WHITE: Color = Color::new(0xFF, 0xFF, 0xFF);
	pub const GENTLE_LILAC: Color = Color::new(0xDD, 0xAA, 0xFF);
	pub const EMU_TURQUOISE: Color = Color::new(0x33, 0xAA, 0x88);
	pub const GREY_DD: Color = Color::new(0xDD, 0xDD, 0xDD);
	pub const GREY_44: Color = Color::new(0x44, 0x44, 0x44);

	pub const fn new(r: u8, g: u8, b: u8) -> Self {
		Color { r, g, b }
	}

	pub const fn u32(&self) -> u32 {
		u32::from_be_bytes([0, self.r, self.g, self.b])
	}
}
