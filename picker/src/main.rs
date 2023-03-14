use fluffy::Buffer;
use winit::{
	dpi::PhysicalSize,
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

fn main() {
	println!("HI :D");

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_inner_size(PhysicalSize::new(640, 480))
		.with_title("picker")
		.build(&event_loop)
		.unwrap();

	let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
	let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

	let mut buffer = Buffer::new(0, 0);

	// it panic if we don't do this.
	buffer.resize_from_physical(window.inner_size());

	println!("Initialized");

	event_loop.run(move |event, _, flow| {
		*flow = ControlFlow::Wait;

		match event {
			Event::RedrawRequested(_wid) => {
				println!("{} {}x{}", buffer.data.len(), buffer.width, buffer.height);
				surface.set_buffer(&buffer.data, buffer.width as u16, buffer.height as u16);
			}

			Event::MainEventsCleared => {}

			Event::WindowEvent {
				event: WindowEvent::Resized(phys),
				..
			} => {
				println!("Reize");
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
				//window.request_redraw();
			}

			_ => (),
		}
	});
}
