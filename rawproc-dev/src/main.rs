use std::time::{Duration, Instant};

use rawproc::decode;

fn main() {
	let name = std::env::args()
		.nth(1)
		.unwrap_or("../rawproc/tests/raw/i_see_you_goose.nef".into());
	let mut p = Profiler::new();
	let mut file = std::fs::File::open(&name).unwrap();

	p.start(Profile::Decode);
	let mut raw = decode(&mut file).unwrap();
	p.end(Profile::Decode);

	p.start(Profile::Crop);
	raw.crop();
	p.end(Profile::Crop);

	p.start(Profile::Whitebalance);
	// Pre bayer whitebalance
	raw.whitebalance();
	p.end(Profile::Whitebalance);

	p.start(Profile::Debayer);
	let rgb = raw.debayer();
	p.end(Profile::Debayer);

	p.start(Profile::XyzToSrgb);
	let xyz = rgb.to_xyz();
	let linsrgb = xyz.to_linsrgb();
	let mut srgb = linsrgb.gamma().floats();
	srgb.contrast(1.1);
	p.end(Profile::XyzToSrgb);

	println!("Decode  {}ms", p.elapsed_ms(Profile::Decode).unwrap());
	println!("Crop    {}ms", p.elapsed_ms(Profile::Crop).unwrap());
	println!("W.B.    {}ms", p.elapsed_ms(Profile::Whitebalance).unwrap());
	println!("Debayer {}ms", p.elapsed_ms(Profile::Debayer).unwrap());
	println!("Colours {}ms", p.elapsed_ms(Profile::XyzToSrgb).unwrap());

	let png_img = srgb.bytes();
	// Write PNG
	let file = std::fs::File::create(std::env::args().nth(2).unwrap()).unwrap();

	let width = png_img.width as u32;
	let height = png_img.height as u32;

	let eight = neam::nearest(&png_img.data, 3, width, height, 1920, 1278);
	let width = 1920;
	let height = 1278;

	let mut enc = png::Encoder::new(file, width, height);
	enc.set_color(png::ColorType::Rgb);
	enc.set_depth(png::BitDepth::Eight);
	enc.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
	let source_chromaticities = png::SourceChromaticities::new(
		(0.31270, 0.32900),
		(0.64000, 0.33000),
		(0.30000, 0.60000),
		(0.15000, 0.06000),
	);
	enc.set_source_chromaticities(source_chromaticities);
	enc.set_srgb(png::SrgbRenderingIntent::Perceptual);

	let mut writer = enc.write_header().unwrap();
	writer.write_image_data(&eight).unwrap();
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
	XyzToSrgb,
}
