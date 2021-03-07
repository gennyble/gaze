mod cli;
mod subsample;

use cli::{CliArgs, OneOrThree};
use image::ImageBuffer;
use image::Rgb as ImageRgb;
use rawproc::{
    debayer::{Debayer, Interpolate, NearestNeighbor},
    image::{Hsv, Image, Rgb, Sensor},
    Processor,
};
use std::path::PathBuf;

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

    let imgbuf: ImageBuffer<ImageRgb<u8>, Vec<u8>> =
        ImageBuffer::from_raw(bytes.meta.width, bytes.meta.height, bytes.data).unwrap();
    imgbuf.save_with_format(out_file, cli.out_type).unwrap()
}

fn directory(cli: CliArgs) {
    let threadpool = threadpool::Builder::new().build();

    let contents = std::fs::read_dir(&cli.in_path).expect("Failed to read input directory");

    for entry in contents {
        let entry = entry.expect("Failed reading a file");
        let mut filename = PathBuf::from(&entry.file_name());
        filename.set_extension(cli.out_type.extensions_str()[0]);

        let cliclone = cli.clone();

        let mut out_file = cli.out_path.clone();
        out_file.push(filename);

        if entry
            .metadata()
            .expect("Failed getting a files metadata")
            .is_file()
        {
            threadpool.execute(move || {
                file(cliclone, &entry.path(), &out_file);
            })
        }
    }

    threadpool.join();
}

fn process(cli: CliArgs, mut sensor_ints: Image<Sensor, u16>) -> Image<Rgb, u8> {
    black_levels(&mut sensor_ints, cli.black);

    let mut sensor_floats = sensor_ints.to_floats();
    white_balance(&mut sensor_floats, cli.white);
    exposure(&mut sensor_floats, cli.exposure);

    let mut rgb_floats = Debayer::rgb(sensor_floats);
    NearestNeighbor::interpolate(&mut rgb_floats);

    Processor::to_sRGB(&mut rgb_floats);
    Processor::sRGB_gamma(&mut rgb_floats);

	let mut hsv_floats: Image<Hsv, f32> = rgb_floats.into();
	brightness(&mut hsv_floats, cli.brightness);
    saturation(&mut hsv_floats, cli.saturation);

    let rgb_floats: Image<Rgb, f32> = hsv_floats.into();
    let mut rgb_bytes = rgb_floats.to_bytes();
    contrast(&mut rgb_bytes, cli.contrast);

    rgb_bytes
}

fn black_levels(rimg: &mut Image<Sensor, u16>, levels_opt: Option<OneOrThree<u16>>) {
    if let Some(levels) = levels_opt {
        match levels {
            OneOrThree::One(v) => Processor::black_levels(rimg, v, v, v),
            OneOrThree::Three(r, g, b) => Processor::black_levels(rimg, r, g, b),
        }
    } else {
        let black = rimg.meta.colordata.black as u16;
        Processor::black_levels(rimg, black, black, black);
    }
}

fn white_balance(rimg: &mut Image<Sensor, f32>, levels_opt: Option<OneOrThree<f32>>) {
    if let Some(levels) = levels_opt {
        match levels {
            OneOrThree::One(v) => Processor::white_balance(rimg, v, v, v),
            OneOrThree::Three(r, g, b) => Processor::white_balance(rimg, r, g, b),
        }
    } else {
        let mut whites = rimg.meta.colordata.cam_mul;

		// Normalize values to green. This prevents the issue where libraw
		// returns whole numbers around 256 instead of floats.
		if whites[1] != 1.0 {
			println!("Normalizing whitebalance coefficients, green is {}", whites[1]);

			whites[0] /= whites[1];
			whites[2] /= whites[1];
			whites[1] /= whites[1];
		}

        println!(
            "Using whitebalance from camera: {}, {}, {}",
            whites[0], whites[1], whites[2]
        );
        Processor::white_balance(rimg, whites[0], whites[1], whites[2]);
    }
}

fn exposure(rimg: &mut Image<Sensor, f32>, ev_opt: Option<f32>) {
    if let Some(ev) = ev_opt {
        Processor::exposure(rimg, ev);
    }
}

fn saturation(img: &mut Image<Hsv, f32>, sat_opt: Option<f32>) {
    if let Some(sat) = sat_opt {
        Processor::saturation(img, sat);
    }
}

fn contrast(cimg: &mut Image<Rgb, u8>, contrast_opt: Option<f32>) {
    if let Some(contrast) = contrast_opt {
        Processor::contrast(cimg, contrast);
    }
}

fn brightness(cimg: &mut Image<Hsv, f32>, brightness_opt: Option<f32>) {
    if let Some(bright) = brightness_opt {
        Processor::brightness(cimg, bright);
    }
}
