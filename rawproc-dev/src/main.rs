use std::time::{Duration, Instant};

use rawproc::decode;

fn main() {
	let mut p = Profiler::new();
	let mut file = std::fs::File::open("../rawproc/tests/raw/i_see_you_goose.nef").unwrap();

	p.start(Profile::Decode);
	let mut raw = decode(&mut file).unwrap();
	p.end(Profile::Decode);

	p.start(Profile::Crop);
	raw.crop();
	p.end(Profile::Crop);

	p.start(Profile::Whitebalance);
	raw.whitebalance();
	p.end(Profile::Whitebalance);

	p.start(Profile::Debayer);
	let rgb = raw.debayer();
	p.end(Profile::Debayer);

	println!("Decode  {}ms", p.elapsed_ms(Profile::Decode).unwrap());
	println!("Crop    {}ms", p.elapsed_ms(Profile::Crop).unwrap());
	println!("W.B.    {}ms", p.elapsed_ms(Profile::Whitebalance).unwrap());
	println!("Debayer {}ms", p.elapsed_ms(Profile::Debayer).unwrap());

	let png_img = rgb;
	// Write PNG
	let file = std::fs::File::create(std::env::args().nth(1).unwrap()).unwrap();
	let mut enc = png::Encoder::new(file, png_img.width as u32, png_img.height as u32);
	enc.set_color(png::ColorType::Rgb);
	enc.set_depth(png::BitDepth::Eight);

	// I want it to be 8bit because sixteen is too big file :(
	let lvl = png_img.metadata.whitelevels[0];
	let eight: Vec<u8> = png_img
		.data
		.into_iter()
		.map(|pix| ((pix as f32 / lvl as f32) * 256.0) as u8)
		.collect();

	/*let scaled = neam::nearest(
		&eight,
		3,
		png_img.width as u32,
		png_img.height as u32,
		1920,
		1278,
	);*/

	let mut writer = enc.write_header().unwrap();
	writer.write_image_data(&eight).unwrap();
}

pub fn float2rgbe(r: f32, g: f32, b: f32) -> [u8; 4] {
	let largest = r.max(g).max(b);
	todo!()
}

struct Profiler {
	start: Vec<(Profile, Instant)>,
	end: Vec<(Profile, Instant)>,
}

impl Profiler {
	pub fn new() -> Self {
		Self {
			start: vec![],
			end: vec![],
		}
	}

	pub fn start(&mut self, prof: Profile) {
		self.start.push((prof, Instant::now()));
	}

	pub fn end(&mut self, prof: Profile) {
		self.end.push((prof, Instant::now()));
	}

	pub fn elapsed(&self, prof: Profile) -> Option<Duration> {
		let start = self.start.iter().find(|(start, _)| *start == prof);
		let end = self.end.iter().find(|(end_prof, _)| *end_prof == prof);

		match start {
			None => None,
			Some((_, time)) => {
				let end = end.map(|(_, time)| time.clone()).unwrap_or(Instant::now());
				Some(end.duration_since(*time))
			}
		}
	}

	pub fn elapsed_ms(&self, prof: Profile) -> Option<u128> {
		self.elapsed(prof).map(|dur| dur.as_millis())
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Profile {
	Decode,
	Crop,
	Whitebalance,
	Debayer,
}
