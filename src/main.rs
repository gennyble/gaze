mod cli;
mod subsample;
#[cfg(feature = "tui")]
mod tui;

use cli::CliArgs;
use image::imageops::rotate270;
use image::imageops::rotate90;
use image::ImageBuffer;
use image::Rgb as ImageRgb;
use rawproc::debayer::{Debayer, Interpolation};
use rawproc::image::HsvImage;
use rawproc::image::RgbImage;
use rawproc::image::SensorImage;
use std::fs::File;
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
        ImageBuffer::from_raw(bytes.meta.width, bytes.meta.height, bytes.data).unwrap();

    let imgbuf = rotate270(&imgbuf);

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
        out_file.push(&filename);

        if entry
            .metadata()
            .expect("Failed getting a files metadata")
            .is_file()
        {
            //threadpool.execute(move || {
            println!("{}", filename.to_string_lossy());
            file(cliclone, &entry.path(), &out_file);
            //})
        }
    }

    threadpool.join();
}

fn process(cli: CliArgs, mut sensor_ints: SensorImage<u16>) -> RgbImage<u8> {
    sensor_ints.black_levels(cli.black.map(|or3| or3.as_triple_tuple()));

    let mut sensor_floats = sensor_ints.into_floats();

    if cli.auto_level {
        let mut lowest = 1.0;
        let mut highest = 0.0;
        for value in sensor_floats.data.iter() {
            if *value < lowest {
                lowest = *value;
            } else if *value > highest {
                highest = *value;
            }
        }

        let high_adjust = 1.0 / (highest - lowest);

        println!("auto-leveling with an ev of {}", high_adjust);

        for value in sensor_floats.data.iter_mut() {
            *value = (*value - lowest) * high_adjust;
        }
    }

    sensor_floats.white_balance(cli.white.map(|or3| or3.as_triple_tuple()));

    if let Some(ev) = cli.exposure {
        sensor_floats.exposure(ev);
    }

    if let Some(curve_file) = cli.tone_curve_path {
        let curve_string = std::fs::read_to_string(curve_file).unwrap();
        let curve_floats: Vec<f32> = curve_string
            .lines()
            .map(|line| line.trim().parse::<f32>().unwrap())
            .collect();

        for pixel in sensor_floats.data.iter_mut() {
            let position = *pixel * (curve_floats.len() as f32 - 1.0);
            let start = curve_floats[position.floor() as usize];
            let end = curve_floats[position.ceil() as usize];
            let percent = position.fract();

            *pixel = lerp(start, end, percent);
        }
    }

    let debayer = Debayer::new(sensor_floats);
    let mut rgb_floats = debayer.interpolate(Interpolation::Bilinear);

    rgb_floats.to_srgb();

    let mut hsv_floats = rgb_floats.into_hsv();
    if let Some(bright) = cli.brightness {
        hsv_floats.brightness(bright);
    }

    if let Some(sat) = cli.saturation {
        hsv_floats.saturation(sat);
    }

    if let Some(shift) = cli.hue_shift {
        hsv_floats.hue_shift(shift);
    }

    let mut rgb_floats = hsv_floats.into_rgb();
    if let Some(con) = cli.contrast {
        rgb_floats.contrast(con);
    }

    rgb_floats.into_u8s()
}

fn lerp(start: f32, end: f32, percent: f32) -> f32 {
    start + (end - start) * percent
}

fn temperature_to_rgb(kelvin: f32) -> (f32, f32, f32) {
    let temperature = kelvin / 100.0;

    let red = if temperature <= 66.0 {
        255.0
    } else {
        let mut red = temperature - 60.0;
        red = 329.698727446 * red.powf(-0.1332047592);

        red.clamp(0.0, 255.0)
    };

    let green = if temperature <= 66.0 {
        let mut green = temperature;
        green = 99.4708025861 * green.ln() - 161.1195681661;

        green.clamp(0.0, 255.0)
    } else {
        let mut green = temperature - 60.0;
        green = 288.1221695283 * green.powf(-0.0755148492);

        green.clamp(0.0, 255.0)
    };

    let blue = if temperature >= 66.0 {
        255.0
    } else {
        if temperature <= 19.0 {
            0.0
        } else {
            let mut blue = temperature - 10.0;
            blue = 138.5177312231 * blue.ln() - 305.0447927307;

            blue.clamp(0.0, 255.0)
        }
    };

    let max = red.min(green).min(blue);

    (1.0 / (red / max), 1.0 / (green / max), 1.0 / (blue / max))
}
