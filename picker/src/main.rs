use fluffy::{Buffer, Color, FluffyWindow};
use winit::{
	dpi::PhysicalSize,
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

fn main() {
	println!("HI :D");

	let builder = WindowBuilder::new()
		.with_inner_size(PhysicalSize::new(640, 480))
		.with_title("picker");

	let mut fluff = FluffyWindow::build_window(builder);

	// Create test image
	let mut img = Buffer::new(256, 256);
	img.data.iter_mut().enumerate().for_each(|(idx, px)| {
		*px = Color::new((idx % 256) as u8, 0, (idx / 256) as u8).u32();
	});

	println!("Initialized");

	let event_loop = fluff.take_el();
	event_loop.run(move |event, _, flow| {
		*flow = ControlFlow::Wait;

		fluff.common_events(&event, flow);

		match event {
			Event::RedrawRequested(_wid) => {
				/*
					Woah woah woah, okay. Hey gen! how do we want this to work?

					Well, I think this is how:
					The user can "window select" an area that they want to fill the screen. We then
					have to find the side that is the least different (division?) from it's
					associated dimension, width/height, and scale both sides by that amount. So then
					we have an area to display, great! But it will almost certainly differ from the
					resolution of the window, so we should scale with neam. Neam here seems alright,
					right? Because we *want* to only have pixels that are in the image. No changing
					pixel values AT ALL!!! And the strange artifacting we get sometimes seems again
					arlight as I really expect users to select into an area, fill the screen, and
					then take a value from there. And we only relly get the artifacting when
					downscaling, anyway.

					But but but! We need a way to look into a rectangular slice of the image. the
					`imgbuffer` (`imgbuf?`) crate does this, but I'm not a great fan of that.

					Hey now, I know we said that we'd try to use existing solutions from crates.io
					more than we've been known for, but but! it's fine... okay? okay. right? right.

					Okay, thanks! Love you <3

					ADENDUM

					Window picking is seemingly very hard to reason about right now. Maybe it's just
					my brainstate. So! So so so *so so* **so so so so** ***so***. We're changing pace
					just a little bit. Kind of! Gently. Gentle pace change.
				*/

				fluff.draw_buffer();
			}

			Event::WindowEvent {
				event: WindowEvent::KeyboardInput { input, .. },
				..
			} => {
				fluff.window.request_redraw();
			}

			_ => (),
		}
	});
}

pub fn fit_image(target_dimensiosn:) {

}