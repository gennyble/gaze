use std::{
	io::Write,
	time::{Duration, Instant},
};

use rawproc::debayer::{Debayer, Interpolation};

const INPUT: &'static str = "tests/raw/i_see_you_goose.nef";
const COUNT: usize = 10;

fn main() {
	let interpolation = match std::env::args().nth(1).as_deref() {
		Some("nearest") => {
			println!("Debayering using Nearest Neighbor");
			Interpolation::NearestNeighbor
		}
		Some("bilinear") => {
			println!("Debayering using Bilinear");
			Interpolation::Bilinear
		}
		_ => panic!(),
	};

	let sensor_floats = rawproc::read_file(INPUT).into_floats();

	let mut times = [Duration::default(); COUNT];

	for idx in 0..COUNT {
		print!("{}", idx);
		std::io::stdout().flush().unwrap();

		let cloned_sensor = sensor_floats.clone();
		let debayerer = Debayer::new(cloned_sensor);

		let before = Instant::now();
		debayerer.interpolate(interpolation);
		times[idx] = before.elapsed();
	}
	println!("");

	times.sort();
	let min = times.iter().skip(1).take(COUNT - 2).min().unwrap();
	let max = times.iter().skip(1).take(COUNT - 2).max().unwrap();
	let average = times.iter().skip(1).take(COUNT - 2).sum::<Duration>() / (COUNT as u32 - 2);

	println!(
		"Ignoring lowest and highest. Counting {} runs out of {}:\n\tMin {}ms\n\tMax {}ms\n\tAverage {}ms",
		COUNT - 2,
		COUNT,
		min.as_millis(),
		max.as_millis(),
		average.as_millis()
	);
}
