mod cli;
mod subsample;
#[cfg(feature = "tui")]
mod tui;

use cli::CliArgs;
use image::ImageBuffer;
use image::Rgb as ImageRgb;
use rawproc::debayer::{Debayer, Interpolation};
use rawproc::image::HsvImage;
use rawproc::image::RgbImage;
use rawproc::image::SensorImage;
use std::path::PathBuf;

fn main() {
    let cli = match CliArgs::new() {
        Ok(cli) => cli,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    #[cfg(feature = "tui")]
    if cli.tui {
        // If the TUI flag is present, go directly there
        panic!("TUI")
    }

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
        ImageBuffer::from_raw(bytes.meta.width, bytes.meta.height, bytes.inner.data).unwrap();
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

fn process(cli: CliArgs, mut sensor_ints: SensorImage<u16>) -> RgbImage<u8> {
    sensor_ints.black_levels(cli.black.map(|or3| or3.as_triple_tuple()));

    let mut sensor_floats = sensor_ints.into_floats();

    sensor_floats.white_balance(cli.white.map(|or3| or3.as_triple_tuple()));

    if let Some(ev) = cli.exposure {
        sensor_floats.exposure(ev);
    }

    let debayer = Debayer::new(sensor_floats);
    let mut rgb_floats = debayer.interpolate(Interpolation::Bilinear);

    rgb_floats.to_srgb();

    let mut hsv_floats: HsvImage<f32> = rgb_floats.into();
    if let Some(bright) = cli.brightness {
        hsv_floats.brightness(bright);
    }

    if let Some(sat) = cli.saturation {
        hsv_floats.saturation(sat);
    }

    if let Some(shift) = cli.hue_shift {
        hsv_floats.hue_shift(shift);
    }

    let mut rgb_floats: RgbImage<f32> = hsv_floats.into();
    if let Some(con) = cli.contrast {
        rgb_floats.contrast(con);
    }

    let rgb_bytes = rgb_floats.into_u8s();

    rgb_bytes
}
