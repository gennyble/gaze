use std::{fs::File, io::Write, path::Path};

// What a great name
pub struct OutImage {
	width: usize,
	height: usize,
	data: Vec<u8>,
}

impl OutImage {
	/// Make a new Image for Output. If your passed in data is not
	/// `width * height * 3` bytes long, this function will panic.
	pub fn new(width: usize, height: usize, data: Vec<u8>) -> Self {
		if data.len() != width * height * 3 {
			// Spitting out the square root seems useful 'cause it's a rough
			// estimation of the dimensions
			panic!(
				"Image dimension are {width}x{height} but data len was {}, ({:02} sqrted)",
				data.len(),
				(data.len() as f32).sqrt()
			)
		} else {
			Self {
				width,
				height,
				data,
			}
		}
	}

	/// Output the image as a PNG. RGB 8bit depth.
	// TODO: gen- no more unwrap!
	pub fn png<P: AsRef<Path>>(&self, path: P) {
		let file = File::create(path.as_ref()).unwrap();
		let mut enc = png::Encoder::new(file, self.width as u32, self.height as u32);
		enc.set_color(png::ColorType::Rgb);
		enc.set_depth(png::BitDepth::Eight);

		let mut writer = enc.write_header().unwrap();
		writer.write_image_data(&self.data).unwrap()
	}

	/// Output the image as a JPEG with the provided quality. RGB 8bit depth.
	// TODO: gen- Fix panic. mozjpeg will panic if it's unhappy and we should
	// catch_unwind and return a result
	pub fn jpeg<P: AsRef<Path>>(&self, path: P, quality: f32) {
		let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);

		comp.set_size(self.width, self.height);
		comp.set_quality(quality);
		comp.set_mem_dest();
		comp.start_compress();
		assert!(comp.write_scanlines(&self.data[..]));

		comp.finish_compress();

		let mut file = File::create(path.as_ref()).unwrap();
		file.write_all(&comp.data_as_mut_slice().unwrap()).unwrap();
	}

	/// Output the image as a lossy WebP with the provided quality.
	// TODO: gen- no more unwrap :)
	pub fn webp<P: AsRef<Path>>(&self, path: P, quality: f32) {
		let enc = webp::Encoder::from_rgb(&self.data, self.width as u32, self.height as u32);
		let img = enc.encode(quality);

		let mut file = File::create(path.as_ref()).unwrap();
		file.write_all(&img).unwrap();
	}
}
