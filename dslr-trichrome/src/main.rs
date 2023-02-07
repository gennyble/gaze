use std::{
	fs::File,
	io::{BufReader, Write},
};

use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use exif::{Field, In, Tag};
use rand::{thread_rng, Rng};
use rawproc::{
	colorspace::{BayerRgb, LinRgb, LinSrgb, Srgb},
	decode,
	image::Image,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Trichrome {
	#[arg(short, long)]
	directory: Option<Utf8PathBuf>,
	#[arg(short, long)]
	red: Option<Utf8PathBuf>,
	#[arg(short, long)]
	green: Option<Utf8PathBuf>,
	#[arg(short, long)]
	blue: Option<Utf8PathBuf>,
	#[arg(short, long)]
	exif: bool,
	#[arg(short = 'k', long)]
	bracketed: bool,
	#[arg(short, long)]
	output_prefix: Utf8PathBuf,
	#[arg(short = 's', long)]
	output_set: bool,
}

impl Trichrome {
	pub fn any_channel_set(&self) -> bool {
		self.red.is_some() || self.green.is_some() || self.blue.is_some()
	}

	pub fn all_channels_set(&self) -> Option<(&Utf8Path, &Utf8Path, &Utf8Path)> {
		match (
			self.red.as_deref(),
			self.green.as_deref(),
			self.blue.as_deref(),
		) {
			(None, _, _) | (_, None, _) | (_, _, None) => None,
			(Some(red), Some(green), Some(blue)) => Some((red, green, blue)),
		}
	}
}

enum Exposures {
	Directory {
		files: Vec<Utf8PathBuf>,
	},
	Explicit {
		red: Utf8PathBuf,
		green: Utf8PathBuf,
		blue: Utf8PathBuf,
	},
}

fn main() {
	let args = Trichrome::parse();

	let exposures = if let Some(path) = args.directory.as_deref() {
		if args.any_channel_set() {
			eprintln!("Only directory OR the three channels may be set");
			return;
		} else {
			Exposures::Directory {
				files: gather_files(path).unwrap(),
			}
		}
	} else if args.any_channel_set() {
		if args.directory.is_some() {
			eprintln!("Only directory OR the three channels may be set");
			return;
		}

		let (red, green, blue) = if let Some(tuple) = args.all_channels_set() {
			tuple
		} else {
			eprintln!("All of the channels must be set");
			return;
		};

		Exposures::Explicit {
			red: red.to_owned(),
			green: green.to_owned(),
			blue: blue.to_owned(),
		}
	} else {
		eprintln!("You have to set the channels (-r, -g, -b) or directory (-d)");
		return;
	};

	if args.exif {
		print_exif(exposures);
		return;
	}

	// We shold make a separete "Explicit" struct instead of just panicing on Directory from here on out
	let explicit = if let Exposures::Directory { files } = exposures {
		let red = files
			.iter()
			.find(|fname| fname.file_stem().unwrap().to_lowercase() == "red");
		let blue = files
			.iter()
			.find(|fname| fname.file_stem().unwrap().to_lowercase() == "green");
		let green = files
			.iter()
			.find(|fname| fname.file_stem().unwrap().to_lowercase() == "blue");

		match (red, green, blue) {
			(None, _, _) => {
				eprintln!("Failed to find red channel image.");
				return;
			}
			(_, None, _) => {
				eprintln!("Failed to find green channel image.");
				return;
			}
			(_, _, None) => {
				eprintln!("Failed to find blue channel image.");
				return;
			}
			(Some(r), Some(g), Some(b)) => Exposures::Explicit {
				red: r.to_owned(),
				green: g.to_owned(),
				blue: b.to_owned(),
			},
		}
	} else {
		exposures
	};

	if args.bracketed {
		println!("Bracketed");
		bracketed(explicit, args.output_prefix, args.output_set)
	} else {
		println!("Trichrome (not bracketed");
		trichrome(explicit, args.output_prefix, args.output_set)
	}
}

fn gather_files(path: &Utf8Path) -> Result<Vec<Utf8PathBuf>, std::io::Error> {
	let mut files = vec![];

	for file in path.read_dir_utf8()? {
		let entry = file?;

		if entry.file_type()?.is_file() {
			files.push(entry.path().to_owned());
		}
	}

	Ok(files)
}

fn print_exif(exposures: Exposures) {
	match exposures {
		Exposures::Directory { files } => {
			for file in files {
				if let Err(e) = print_file_exif(&file) {
					eprintln!("Failed {file}: {e}");
					continue;
				}
			}
		}
		Exposures::Explicit { red, green, blue } => {
			print_file_exif(red).unwrap();
			print_file_exif(green).unwrap();
			print_file_exif(blue).unwrap();
		}
	}
}

fn print_file_exif<P: AsRef<Utf8Path>>(path: P) -> Result<(), exif::Error> {
	let path = path.as_ref();
	let file = File::open(&path)?;
	let mut bufread = BufReader::new(file);
	let exifreader = exif::Reader::new();
	let exif = exifreader.read_from_container(&mut bufread)?;

	let aperture = exif.get_field(Tag::ApertureValue, In::PRIMARY);
	let focal = exif.get_field(Tag::FocalLength, In::PRIMARY);
	let eidx = exif.get_field(Tag::ExposureIndex, In::PRIMARY);
	let sense = exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY);
	let sense_type = exif.get_field(Tag::SensitivityType, In::PRIMARY);
	let etime = exif.get_field(Tag::ExposureTime, In::PRIMARY);
	let fnum = exif.get_field(Tag::FNumber, In::PRIMARY);

	let exif_print = |name: &'static str, field: Option<&Field>| {
		print!("\t{name} ");
		if let Some(field) = field {
			print!("{}", field.display_value())
		} else {
			print!("none")
		}
		print!("\n")
	};

	println!("File {path}");
	exif_print("Aperture", aperture);
	exif_print("Focal Length", focal);
	exif_print("Exposure Index", eidx);
	exif_print("Sense", sense);
	exif_print("Exposure Time", etime);
	exif_print("F Number", fnum);

	Ok(())
}

fn bracketed(exposures: Exposures, prefix: Utf8PathBuf, set: bool) {
	let get_raw = |path: &Utf8Path| -> Image<u16, BayerRgb> {
		let mut file = File::open(path).unwrap();
		decode(&mut file).unwrap()
	};

	let (red, green, blue) = match exposures {
		Exposures::Directory { .. } => panic!(),
		Exposures::Explicit { red, green, blue } => (red, green, blue),
	};

	let mut red = get_raw(&red);
	let mut green = get_raw(&green);
	let mut blue = get_raw(&blue);

	red.crop();
	green.crop();
	blue.crop();

	let mut rgb = trichrome_debayer(red, green, blue);

	// Incrasing exposure (this also seems to clip the white square on the color
	// chart exposures, making it look whiter than it is, oops)
	let lv = rgb.metadata.whitelevels[0];
	for light in rgb.data.iter_mut() {
		*light =
			(((*light as f32 / lv as f32) * 2f32.powf(2.0)).clamp(0.0, 1.0) * lv as f32) as u16;
	}

	// I'm just transforing the colorspace here so I can get access to the gamma
	let lsrgb = Image::<_, LinSrgb>::from_raw_parts(rgb.width, rgb.height, rgb.metadata, rgb.data);
	let srgb = lsrgb.gamma();

	let tri: TrichromedImage = srgb.into();
	if set {
		tri.output_set("bracketed", prefix);
	} else {
		tri.png(format!("{prefix}_bracketed.png"));
	}
}

fn trichrome(exposures: Exposures, prefix: Utf8PathBuf, set: bool) {
	let get_raw = |path: &Utf8Path| -> Image<u16, BayerRgb> {
		let mut file = File::open(path).unwrap();
		decode(&mut file).unwrap()
	};

	let (red, green, blue) = match exposures {
		Exposures::Directory { .. } => panic!(),
		Exposures::Explicit { red, green, blue } => (red, green, blue),
	};

	let mut red = get_raw(&red);
	red.crop();
	red.whitebalance();

	let mut green = get_raw(&green);
	green.crop();
	green.whitebalance();

	let mut blue = get_raw(&blue);
	blue.crop();
	blue.whitebalance();

	let mut rgb = trichrome_debayer(red, green, blue);

	// Incrasing exposure
	let lv = rgb.metadata.whitelevels[0];
	for light in rgb.data.iter_mut() {
		*light =
			(((*light as f32 / lv as f32) * 2f32.powf(2.0)).clamp(0.0, 1.0) * lv as f32) as u16;
	}

	let linsrgb = rgb.to_xyz().to_linsrgb();
	let srgb = linsrgb.gamma();

	let tri: TrichromedImage = srgb.into();
	if set {
		tri.output_set("trichrome", prefix);
	} else {
		tri.png(format!("{prefix}_trichrome.png"));
	}
}

type Raw = Image<u16, BayerRgb>;
fn trichrome_debayer(r: Raw, g: Raw, b: Raw) -> Image<u16, LinRgb> {
	let mut rgb = vec![0; g.width * g.height * 3];

	// We use g as the reference throughout. It doesn't *really* matter which
	// raw we get these values from, but green is probably best because a lot
	// of the time it's the "standard" in the white balance. I mean, the other
	// values are normalized according to green
	let cfa = g.metadata.cfa.clone();
	let mut rr = RollingRandom::new();

	#[rustfmt::skip]
	let options = [
		(-1, -1), (0, -1), (1, -1),
		(-1, 0),  /*skip*/ (1, 0),
		(-1, 1),  (0, 1),  (1, 1)
	];

	let get = |raw: &Raw, p: (usize, usize)| -> u16 { raw.data[raw.width * p.1 + p.0] };
	let mut set = |x: usize, y: usize, color: usize, v: u16| {
		rgb[(g.width * y + x) * 3 + color] = v;
	};

	for x in 1..g.width - 1 {
		for y in 1..g.height - 1 {
			let options = options.clone().into_iter().map(|(x_off, y_off)| {
				let x = (x as isize + x_off) as usize;
				let y = (y as isize + y_off) as usize;
				(cfa.color_at(x, y), x, y)
			});

			match cfa.color_at(x, y) {
				0 => {
					// Red
					set(x, y, 0, get(&r, (x, y)));
					// Green
					set(x, y, 1, get(&g, pick_color(&mut rr, options.clone(), 1)));
					// Blue
					set(x, y, 2, get(&b, pick_color(&mut rr, options, 2)));
				}
				1 => {
					// Green
					set(x, y, 1, get(&g, (x, y)));
					// Red
					set(x, y, 0, get(&r, pick_color(&mut rr, options.clone(), 0)));
					// Blue
					set(x, y, 2, get(&b, pick_color(&mut rr, options, 2)));
				}
				2 => {
					// Blue
					set(x, y, 2, get(&b, (x, y)));
					// Red
					set(x, y, 0, get(&r, pick_color(&mut rr, options.clone(), 0)));
					// Green
					set(x, y, 1, get(&g, pick_color(&mut rr, options, 1)));
				}
				_ => unreachable!(),
			}
		}
	}

	Image::from_raw_parts(g.width, g.height, g.metadata, rgb)
}

#[inline]
fn pick_color<I>(roll: &mut RollingRandom, options: I, color: usize) -> (usize, usize)
where
	I: Iterator<Item = (usize, usize, usize)>,
{
	let colors: Vec<(usize, usize, usize)> = options.filter(|(clr, _, _)| *clr == color).collect();
	let random = roll.random_u8() % colors.len() as u8;
	let picked = &colors[random as usize];

	(picked.1, picked.2)
}

// I carry this struct around with me a lot, lol. I should put it in a crate, but it
// almost doesn't feel worth it. It's a fast random thing because generating random
// values is slow. This just allocates a big ol' array and fills it all at once so
// we can use them at will. It wraps around, but it should be enough random to not
// generate a pattern? maybe???
struct RollingRandom {
	values: [u8; Self::BUCKET_SIZE],
	index: u16,
}

impl RollingRandom {
	// 8.3K
	// .3 because 4K is a common image length, and I'd rather if there is a pattern
	// that it fall on a diagonal I guess?
	const BUCKET_SIZE: usize = 8533;

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

// Oh yeah, this is a great name.
struct TrichromedImage {
	width: usize,
	height: usize,
	data: Vec<u8>,
}

impl TrichromedImage {
	// Make a half-sized image using neam to downscale
	pub fn half(&self) -> TrichromedImage {
		// The div by four then scale by 2 is a cheap way to get even dimesnions,
		// which I very much want for some reason
		let width = (self.width / 4) * 2;
		let height = (self.height / 4) * 2;
		let data = neam::nearest(
			&self.data,
			3,
			self.width as u32,
			self.height as u32,
			width as u32,
			height as u32,
		);

		Self {
			width,
			height,
			data,
		}
	}

	// Save as a PNG
	pub fn png<P: AsRef<Utf8Path>>(&self, path: P) {
		let file = std::fs::File::create(path.as_ref()).unwrap();
		let mut enc = png::Encoder::new(file, self.width as u32, self.height as u32);
		enc.set_color(png::ColorType::Rgb);
		enc.set_depth(png::BitDepth::Eight);

		let mut writer = enc.write_header().unwrap();
		writer.write_image_data(&self.data).unwrap();
	}

	pub fn jpeg<P: AsRef<Utf8Path>>(&self, path: P, quality: f32) {
		let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);

		comp.set_size(self.width, self.height);
		comp.set_quality(quality);
		comp.set_mem_dest();
		comp.start_compress();
		assert!(comp.write_scanlines(&self.data[..]));

		comp.finish_compress();

		let mut file = File::create(path.as_ref()).unwrap();
		file.write_all(&comp.data_to_vec().unwrap()).unwrap();
	}

	pub fn webp<P: AsRef<Utf8Path>>(&self, path: P, quality: f32) {
		let enc = webp::Encoder::from_rgb(&self.data, self.width as u32, self.height as u32);
		let img = enc.encode(quality);

		let mut file = File::create(path.as_ref()).unwrap();
		file.write_all(&img).unwrap();
	}

	pub fn output_set<P: Into<Utf8PathBuf>, S: AsRef<str>>(&self, dir_name: P, prefix: S) {
		let prefix = prefix.as_ref();
		let dir_name = dir_name.into();

		if !dir_name.exists() {
			std::fs::create_dir(&dir_name).unwrap();
		}

		let make_path =
			|size: &'static str, ext: &'static str| dir_name.join(format!("{prefix}_{size}.{ext}"));

		self.png(make_path("full", "png"));
		let half = self.half();
		half.jpeg(make_path("half", "jpg"), 50.0);
		half.webp(make_path("half", "webp"), 45.0);
		let quart = half.half();
		quart.jpeg(make_path("quarter", "jpg"), 80.0);
		quart.webp(make_path("quarter", "webp"), 75.0);
	}
}

impl From<Image<u16, Srgb>> for TrichromedImage {
	fn from(srgb: Image<u16, Srgb>) -> Self {
		// yes I should use the proper whitelevel per channel. but! i do not care!
		// they're all the same for my camera, and you're too i'd imagine
		let lvl = srgb.metadata.whitelevels[0];
		let eight: Vec<u8> = srgb
			.data
			.into_iter()
			.map(|pix| ((pix as f32 / lvl as f32) * 255.0) as u8)
			.collect();

		Self {
			width: srgb.width,
			height: srgb.height,
			data: eight,
		}
	}
}
