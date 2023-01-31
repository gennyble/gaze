use rawproc::decode;

fn main() {
	let mut file = std::fs::File::open("../rawproc/tests/raw/i_see_you_goose.nef").unwrap();
	let mut raw = decode(&mut file).unwrap();

	raw.crop();

	// Write PNG
	let file = std::fs::File::create(std::env::args().nth(1).unwrap()).unwrap();
	let mut enc = png::Encoder::new(file, raw.width as u32, raw.height as u32);
	enc.set_color(png::ColorType::Grayscale);
	enc.set_depth(png::BitDepth::Eight);

	// I want it to be 8bit because sixteen is too big file :(
	let lvl = raw.colorspace.metadata.whitelevels[0];
	let eight: Vec<u8> = raw
		.data
		.into_iter()
		.map(|pix| ((pix as f32 / lvl as f32) * 256.0) as u8)
		.collect();

	let mut writer = enc.write_header().unwrap();
	writer.write_image_data(&eight).unwrap();
}

pub fn float2rgbe(r: f32, g: f32, b: f32) -> [u8; 4] {
	let largest = r.max(g).max(b);
	todo!()
}
