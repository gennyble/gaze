use std::num::NonZeroUsize;

use flo_curves::{
	bezier::{BezierCurve2D, Curve},
	BezierCurve, BezierCurveFactory, Coord2,
};
use winit::{
	dpi::PhysicalSize,
	event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

fn main() {
	println!("Hi! :D");

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_inner_size(PhysicalSize::new(640, 480))
		.with_title("curver")
		.build(&event_loop)
		.unwrap();

	let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
	let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

	let mut buffer = Buffer::new(0, 0);

	let mut c1 = (170.0, 670.0);
	let mut c2 = (830.0, 670.0);

	// Will it even have an inner size yet? I'm pretty sure we get a resize in
	// the first bunch of events, so it doesn't matter. but might as well try?
	buffer.resize_from_physical(window.inner_size());

	event_loop.run(move |event, _, flow| {
		*flow = ControlFlow::Wait;

		match event {
			Event::RedrawRequested(_wid) => {
				surface.set_buffer(&buffer.data, buffer.width as u16, buffer.height as u16)
			}

			Event::WindowEvent {
				event: WindowEvent::Resized(phys),
				..
			} => {
				buffer.resize_from_physical(phys);
				window.request_redraw();
			}

			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => {
				*flow = ControlFlow::Exit;
			}

			Event::WindowEvent {
				event: WindowEvent::KeyboardInput { input, .. },
				..
			} => {
				if let Some(VirtualKeyCode::R) = input.virtual_keycode {
					draw(&mut buffer, c1, c2);
					window.request_redraw();
				}
			}

			_ => (),
		}
	});
}

fn draw(buf: &mut Buffer, c1: (f64, f64), c2: (f64, f64)) {
	buf.clear();

	let curve = Curve::from_points(
		Coord2(0.0, 0.0),
		(Coord2(c1.0, c1.1), Coord2(c2.0, c2.1)),
		Coord2(1000.0, 1000.0),
	);

	let width = buf.width;
	let height = buf.height;

	for t in 0..width {
		let normal_t = t as f64 / width as f64;
		let point = curve.point_at_pos(normal_t);

		let normal_x = point.0 / 1000.0;
		let normal_y = 1.0 - (point.1 / 1000.0);

		let x = (normal_x * width as f64 - 1.0).floor() as usize;
		let y = (normal_y * height as f64 - 1.0).floor() as usize;

		//buf.set_unchecked(x, y, 0xFF, 0xFF, 0xFF);
		buf.rect(
			x.saturating_add(1),
			y.saturating_add(1),
			3,
			3,
			0xFF,
			0xFF,
			0xFF,
		)
	}

	let c1 = buf.dethou(c1);

	buf.rect(
		c1.0.saturating_add(5),
		(height - c1.1).saturating_add(5),
		20,
		20,
		0xDD,
		0xAA,
		0xFF,
	);

	let c2 = buf.dethou(c2);

	buf.rect(
		c2.0.saturating_add(5),
		(height - c2.1).saturating_add(5),
		20,
		20,
		0xDD,
		0xAA,
		0xFF,
	);
}

struct Buffer {
	/// Bytes - 0RGB
	data: Vec<u32>,
	width: usize,
	height: usize,
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
	pub fn set(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
		if y >= self.height || x >= self.width {
			return;
		}

		self.set_unchecked(x, y, r, g, b)
	}

	pub fn set_unchecked(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
		let px = &mut self.data[y * self.width + x];
		*px = u32::from_be_bytes([0x0, r, g, b]);
	}

	pub fn rect(&mut self, x: usize, y: usize, width: usize, height: usize, r: u8, g: u8, b: u8) {
		for px in x..x + width {
			for py in y..y + height {
				self.set(px, py, r, g, b)
			}
		}
	}
}
