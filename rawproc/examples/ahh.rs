use rawproc::decode;

fn main() {
	let mut file = std::fs::File::open("tests/raw/i_see_you_goose.nef").unwrap();
	let mut raw = decode(&mut file).unwrap();

	// Write PNG
	let mut file = std::fs::File::create(std::env::args().nth(1).unwrap()).unwrap();
}

pub fn float2rgbe(r: f32, g: f32, b: f32) -> [u8; 4] {
	let largest = r.max(g).max(b);
	todo!()
}
