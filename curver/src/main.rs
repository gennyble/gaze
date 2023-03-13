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
		.build(&event_loop)
		.unwrap();
	let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
	let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

	let mut buffer = Buffer::new(0, 0);

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
					draw(&mut buffer);
					window.request_redraw();
				}
			}

			_ => (),
		}
	});
}

fn draw(buf: &mut Buffer) {
	buf.clear();

	let curve = Curve::from_points(
		Coord2(0.0, 0.0),
		(Coord2(170.0, 670.0), Coord2(830.0, 670.0)),
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

		buf.set_unchecked(x, y, 0xFF, 0xFF, 0xFF);
	}
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
}
