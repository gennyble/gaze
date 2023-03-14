use flo_curves::{bezier::Curve, BezierCurve, BezierCurveFactory, Coord2};
use fluffy::{Buffer, Color};
use rfd::FileDialog;
use winit::{
	dpi::PhysicalSize,
	event::{ElementState, Event, VirtualKeyCode, WindowEvent},
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
	let mut selected = SelectedPoint::None;
	let mut should_open_file_dialog = false;
	let mut unsaved_changes = false;

	// Will it even have an inner size yet? I'm pretty sure we get a resize in
	// the first bunch of events, so it doesn't matter. but might as well try?
	buffer.resize_from_physical(window.inner_size());

	event_loop.run(move |event, _, flow| {
		*flow = ControlFlow::Wait;

		match event {
			Event::RedrawRequested(_wid) => {
				draw(&mut buffer, c1, c2, selected);
				surface.set_buffer(&buffer.data, buffer.width as u16, buffer.height as u16);
			}

			Event::MainEventsCleared => {
				if should_open_file_dialog {
					should_open_file_dialog = false;

					save_dialog(c1, c2, &mut unsaved_changes);

					if !unsaved_changes {
						window.set_title("curver");
					}
				}
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
				if input.state != ElementState::Pressed {
					return;
				}

				let vkc = match input.virtual_keycode {
					Some(vkc) => vkc,
					None => return,
				};

				if vkc == VirtualKeyCode::R {
					draw(&mut buffer, c1, c2, selected);
				}

				if vkc == VirtualKeyCode::S {
					should_open_file_dialog = true;
				}

				match vkc {
					VirtualKeyCode::Key1 => selected = SelectedPoint::Control1,
					VirtualKeyCode::Key2 => selected = SelectedPoint::Control2,
					VirtualKeyCode::Grave => selected = SelectedPoint::None,
					_ => (),
				}

				let point = match selected {
					SelectedPoint::Control1 => &mut c1,
					SelectedPoint::Control2 => &mut c2,
					_ => return,
				};

				match vkc {
					VirtualKeyCode::Up => point.1 += 10.0,
					VirtualKeyCode::Down => point.1 -= 10.0,
					VirtualKeyCode::Left => point.0 -= 10.0,
					VirtualKeyCode::Right => point.0 += 10.0,
					_ => (),
				}

				if !unsaved_changes {
					window.set_title("curver - unsaved changes");
					unsaved_changes = true;
				}

				window.request_redraw();
			}

			_ => (),
		}
	});
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SelectedPoint {
	None,
	Control1,
	Control2,
}

fn draw(buf: &mut Buffer, c1: (f64, f64), c2: (f64, f64), sel: SelectedPoint) {
	buf.clear();

	let width = buf.width;
	let height = buf.height;

	// Draw the grid!
	let gridc = Color::GREY_44;

	let wh = buf.width / 2;
	let wq = wh / 2;

	buf.vert(wq, 0, height, gridc);
	buf.vert(wh, 0, height, gridc);
	buf.vert(wh + wq, 0, height, gridc);

	let hh = buf.height / 2;
	let hq = hh / 2;

	buf.hori(hq, 0, width, gridc);
	buf.hori(hh, 0, width, gridc);
	buf.hori(hh + hq, 0, width, gridc);

	let curve = Curve::from_points(
		Coord2(0.0, 0.0),
		(Coord2(c1.0, c1.1), Coord2(c2.0, c2.1)),
		Coord2(1000.0, 1000.0),
	);

	for t in 0..width {
		let normal_t = t as f64 / width as f64;
		let point = curve.point_at_pos(normal_t);

		let normal_x = point.0 / 1000.0;
		let normal_y = 1.0 - (point.1 / 1000.0);

		let x = (normal_x * width as f64 - 1.0).floor() as usize;
		let y = (normal_y * height as f64 - 1.0).floor() as usize;

		//buf.set_unchecked(x, y, 0xFF, 0xFF, 0xFF);
		buf.rect(x.saturating_add(1), y.saturating_add(1), 3, 3, Color::WHITE)
	}

	let (c1_color, c2_color) = match sel {
		SelectedPoint::None => (Color::GENTLE_LILAC, Color::GENTLE_LILAC),
		SelectedPoint::Control1 => (Color::EMU_TURQUOISE, Color::GENTLE_LILAC),
		SelectedPoint::Control2 => (Color::GENTLE_LILAC, Color::EMU_TURQUOISE),
	};

	let c1 = buf.dethou(c1);

	buf.rect(
		c1.0.saturating_add(5),
		(height - c1.1).saturating_add(5),
		20,
		20,
		c1_color,
	);

	let c2 = buf.dethou(c2);

	buf.rect(
		c2.0.saturating_add(5),
		(height - c2.1).saturating_add(5),
		20,
		20,
		c2_color,
	);
}

fn save_dialog(c1: (f64, f64), c2: (f64, f64), unsaved: &mut bool) {
	let curve = Curve::from_points(
		Coord2(0.0, 0.0),
		(Coord2(c1.0, c1.1), Coord2(c2.0, c2.1)),
		Coord2(1000.0, 1000.0),
	);

	let mut buffer = String::new();

	// 12bits!
	for line in 0..=4095 {
		let t = line as f64 / 4095.0;
		let y = curve.point_at_pos(t).1 / 1000.0;
		buffer.push_str(&format!("{y:.6}\n"));
	}

	let file = FileDialog::new()
		.add_filter("Line Separated Value", &["lsv", "txt", ""])
		.save_file();

	if let Some(path) = file {
		std::fs::write(path, buffer.as_bytes()).unwrap();
		*unsaved = false;
	}
}
