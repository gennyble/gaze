mod cli;
mod subsample;

use std::path::PathBuf;
use rawproc::{Processor,image::{RawImage, RgbImage}, debayer::{Debayer, Interpolate, NearestNeighbor}};
use image::ImageBuffer;
use image::Rgb;
use image::ImageFormat;
use cli::{CliArgs, OneOrThree};

fn main() {
    let cli = match CliArgs::new() {
        Ok(cli) => cli,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    if cli.in_is_dir {
        directory(cli);
    } else {
        file(cli.clone(), &cli.in_path, &cli.out_path);
    }
}

fn file(cli: CliArgs, in_file: &PathBuf, out_file: &PathBuf) {
    let mut rimg = rawproc::read_file(in_file.to_str().unwrap());

    if cli.thumb {
        rimg = subsample::subsample(rimg);
    }

    let bytes = process(cli.clone(), rimg);

    let imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(bytes.meta.width, bytes.meta.height, bytes.rgb).unwrap();
    imgbuf.save_with_format(out_file, cli.out_type).unwrap()
}

fn directory(cli: CliArgs) {
    let threadpool = threadpool::Builder::new().build();

	let contents = std::fs::read_dir(&cli.in_path).expect("Failed to read input directory");

	for entry in contents {
		let entry = entry.expect("Failed reading a file");
		let mut filename = PathBuf::from(&entry.file_name());
		filename.set_extension("jpg");

        let cliclone = cli.clone();

		let mut out_file = cli.out_path.clone();
		out_file.push(filename);

		if entry.metadata().expect("Failed getting a files metadata").is_file() {
			threadpool.execute(move || {
                file(cliclone, &entry.path(), &out_file);
			})
		}
	}

	threadpool.join();
}

fn process(cli: CliArgs, mut rimg: RawImage) -> RgbImage<u8> {
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

    bytes
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