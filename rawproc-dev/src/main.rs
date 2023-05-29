use std::time::{Duration, Instant};

use imgout::OutImage;
use rawproc::{
	colorspace::{Hsv, Srgb},
	decode,
	image::Image,
};

fn main() {
	let name = std::env::args()
		.nth(1)
		.unwrap_or("../rawproc/tests/raw/i_see_you_goose.nef".into());
	let mut p = Profiler::new();
	let mut file = std::fs::File::open(&name).unwrap();

	p.start(Profile::AllOfIt);
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

	println!("WB {:?}", raw.metadata.whitebalance);

	p.start(Profile::PreColour);
	for px in raw.data.iter_mut() {
		//*px = (*px as f32).powf(1.0) as u16; //(*px as f32 * 6.0) as u16;
		*px = ((*px as f32) * 1.25) as u16;
	}
	p.end(Profile::PreColour);

	p.start(Profile::Debayer);
	let rgb = raw.debayer();
	p.end(Profile::Debayer);

	p.start(Profile::XyzToSrgb);
	let xyz = rgb.to_xyz();
	let linsrgb = xyz.to_linsrgb();
	p.end(Profile::XyzToSrgb);

	let curve_file = "/Users/gen/weird.lsv";
	//let curve_file = "../curve.lsv";
	let curve_string = std::fs::read_to_string(curve_file).unwrap();

	p.start(Profile::ToneCurve);
	let curve_floats: Vec<f32> = curve_string
		.lines()
		.map(|line| line.trim().parse::<f32>().unwrap())
		.collect();

	let mut flinsrgb = linsrgb.floats();
	for pixel in flinsrgb.data.iter_mut() {
		let position = pixel.clamp(0.0, 1.0) * (curve_floats.len() as f32 - 1.0);
		let start = curve_floats[position.floor() as usize];
		let end = curve_floats[position.ceil() as usize];
		let percent = position.fract();

		*pixel = lerp(start, end, percent);
	}
	p.end(Profile::ToneCurve);

	p.start(Profile::Colour);
	let mut srgb = flinsrgb.gamma();
	srgb.contrast(1.05);
	srgb.autolevel();

	let mut hsv: Image<f32, Hsv> = srgb.into();
	hsv.saturation(1.05);
	let srgb: Image<f32, Srgb> = hsv.into();
	p.end(Profile::Colour);
	p.end(Profile::AllOfIt);

	println!("");
	println!("Decode  {}ms", p.elapsed_ms(Profile::Decode).unwrap());
	println!("Crop    {}ms", p.elapsed_ms(Profile::Crop).unwrap());
	println!("W.B.    {}ms", p.elapsed_ms(Profile::Whitebalance).unwrap());
	println!("Colour  {}ms", p.elapsed_ms(Profile::PreColour).unwrap());
	println!("Debayer {}ms\n", p.elapsed_ms(Profile::Debayer).unwrap());
	println!("XYZ->sRGB {}ms", p.elapsed_ms(Profile::XyzToSrgb).unwrap());
	println!("Curve     {}ms", p.elapsed_ms(Profile::ToneCurve).unwrap());
	println!("Colour    {}ms\n", p.elapsed_ms(Profile::Colour).unwrap());
	println!(
		"AllOfIt {:.3}s",
		p.elapsed(Profile::AllOfIt).unwrap().as_secs_f64()
	);

	let img = srgb.bytes();

	let out = OutImage::new(img.width, img.height, img.data);
	let name = std::env::args().nth(2).unwrap();
	out.jpeg(name, 85.0);

	/*let mut enc = png::Encoder::new(file, width, height);
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
	writer.write_image_data(&data).unwrap();*/
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
	AllOfIt,
	Decode,
	Crop,
	Whitebalance,
	PreColour,
	Debayer,
	ToneCurve,
	XyzToSrgb,
	Colour,
}

fn lerp(start: f32, end: f32, percent: f32) -> f32 {
	start + (end - start) * percent
}
