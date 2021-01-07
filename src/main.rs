use getopts::Options;
use std::path::PathBuf;
use std::str::FromStr;
use rawproc::{Processor,image::{RawImage, RgbImage}, debayer::{Debayer, Interpolate, NearestNeighbor}};
use image::ImageBuffer;
use image::Rgb;
use image::ImageFormat;

enum OneOrThree<T> {
    One(T),
    Three(T, T, T)
}

impl<T: FromStr> FromStr for OneOrThree<T> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(',') {
            // Three values should be present

            let values: Vec<&str> = s.split(',').map(|s| s.trim()).collect();
            let v1 = values[0].parse::<T>().map_err(|_e|{})?;
            let v2 = values[0].parse::<T>().map_err(|_e|{})?;
            let v3 = values[0].parse::<T>().map_err(|_e|{})?;

            Ok(OneOrThree::Three(v1, v2, v3))
        } else {
            // One value should be present

            Ok(OneOrThree::One(s.parse::<T>().map_err(|_e|{})?))
        }
    }
}

struct CliArgs {
    pub in_file: PathBuf,
    pub out_file: PathBuf,
    pub black: Option<OneOrThree<u16>>,
    pub white: Option<OneOrThree<f32>>,
    pub exposure: Option<f32>,
    pub contrast: Option<f32>,
    pub brightness: Option<u8>
}

impl CliArgs {
    fn print_usage(program: &str, opts: Options) {
        let brief = format!("Usage: {} FILE [options]", program);
        println!("{}", opts.usage(&brief));
    }

    pub fn new() -> Option<Self> {
        let args: Vec<String> = std::env::args().collect();
        let program = &args[0];
    
        let mut opts = Options::new();
        opts.reqopt("i", "ifile", "Input path to process", "FILE");
        opts.reqopt("o", "ofile", "Output path", "FILE");
        opts.optopt("l", "black", "Black level adjustment values\nDefaults to camera's values\nEx: 150 or 150,200,150", "NUMBERS");
        opts.optopt("w", "white", "White balance adjustment values\nDefaults to camera's values\nEx: 1.0 or 2.1,1.0,1.3", "NUMBERS");
        opts.optopt("e", "exposure", "Exposure compensation value\nEx: 1.2", "NUMBER");
        opts.optopt("c", "contrast", "Contrast adjustment value\nEx: 1.2", "NUMBER");
        opts.optopt("b", "brightness", "Brightness addition\nEx: 10", "NUMBER");
        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(_e) => {
                Self::print_usage(program, opts);
                return None;
            }
        };
    
        let in_file = PathBuf::from(matches.opt_str("ifile").expect("How'd this happen? ifile isn't present"));
        let out_file = PathBuf::from(matches.opt_str("ofile").expect("How'd this happen? ofile isn't present"));
        let black = matches.opt_get("black").expect("Failed to parse black level values");
        let white = matches.opt_get("white").expect("Failed to parse white balance values");
        let exposure = matches.opt_get("exposure").expect("Failed to parse exposure value");
        let contrast = matches.opt_get("contrast").expect("Failed to parse contrast value");
        let brightness = matches.opt_get("brightness").expect("Failed to parse brightness value");

        Some(Self {
            in_file,
            out_file,
            black,
            white,
            exposure,
            contrast,
            brightness
        })
    }
}

fn main() {
    let cli = CliArgs::new().expect("Cli is none?");

    let mut rimg = rawproc::read_file(cli.in_file.to_str().unwrap());
    black_levels(&mut rimg, cli.black);
    white_balance(&mut rimg, cli.white);
    exposure(&mut rimg, cli.exposure);

    let mut cimg = Debayer::rgb(rimg);
    NearestNeighbor::interpolate(&mut cimg);

    let mut floats = cimg.as_float_image();
    Processor::to_sRGB(&mut floats);
    Processor::sRGB_gamma(&mut floats);

    let mut bytes = floats.as_byte_image();
    contrast(&mut bytes, cli.contrast);
    brightness(&mut bytes, cli.brightness);

    let imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(bytes.meta.width, bytes.meta.height, bytes.rgb).unwrap();
    imgbuf.save_with_format(cli.out_file, ImageFormat::Jpeg).unwrap()
}

fn black_levels(rimg: &mut RawImage, levels_opt: Option<OneOrThree<u16>>) {
    if let Some(levels) = levels_opt {
        match levels {
            OneOrThree::One(v) => Processor::black_levels(rimg, v, v, v),
            OneOrThree::Three(r, g, b) => Processor::black_levels(rimg, r, g, b)
        }
    } else {
        let black = rimg.meta.colordata.black as u16;
        Processor::black_levels(rimg, black, black, black);
    }
}

fn white_balance(rimg: &mut RawImage, levels_opt: Option<OneOrThree<f32>>) {
    if let Some(levels) = levels_opt {
        match levels {
            OneOrThree::One(v) => Processor::white_balance(rimg, v, v, v),
            OneOrThree::Three(r, g, b) => Processor::white_balance(rimg, r, g, b)
        }
    } else {
        let whites = rimg.meta.colordata.cam_mul;
        Processor::white_balance(rimg, whites[0], whites[1], whites[2]);
    }
}

fn exposure(rimg: &mut RawImage, ev_opt: Option<f32>) {
    if let Some(ev) = ev_opt {
        Processor::exposure(rimg, ev);
    }
}

fn contrast(cimg: &mut RgbImage<u8>, contrast_opt: Option<f32>) {
    if let Some(contrast) = contrast_opt {
        Processor::contrast(cimg, contrast);
    }
}

fn brightness(cimg: &mut RgbImage<u8>, brightness_opt: Option<u8>) {
    if let Some(bright) = brightness_opt {
        Processor::brightness(cimg, bright);
    }
}

/*
fn process_directory(cli: CliArgs) {
	let threadpool = threadpool::Builder::new().build();

	let before = Instant::now();
	let contents = fs::read_dir(cli.in_file).expect("Failed to read input directory");
	let ev = cli.ev;

	for entry in contents {
		let entry = entry.expect("Failed reading a file");
		let mut filename = PathBuf::from(&entry.file_name());

		let in_file = filename.to_str().unwrap().to_owned();
		filename.set_extension("JPG");

		let mut out_file = PathBuf::from(&cli.out_file);
		out_file.push(filename);
		let out_file = out_file.to_str().unwrap().to_owned();

		if entry.metadata().expect("Failed getting a files metadata").is_file() {
			threadpool.execute(move || {
				let (rawimg, colordata) = decode(entry.path().to_str().unwrap());
				get_rgb(&out_file, rawimg, colordata, ev.clone());
				println!("Finished processing {}, saving as {}", in_file, out_file);
			})
		}
	}

	threadpool.join();
	println!("Finished processing directory in {} seconds", Instant::now().duration_since(before).as_secs());
}*/