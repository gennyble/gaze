use std::{fs::File, thread::JoinHandle, time::Instant};

use egui::{Color32, ColorImage, RichText, TextureHandle, Vec2};
use rawproc::{colorspace::Srgb, image::Image};
use rgb::FromSlice;

pub fn run_gui() -> ! {
	let mut native_options = eframe::NativeOptions::default();
	native_options.max_window_size = Some(Vec2::new(1080.0, 720.0));
	native_options.min_window_size = Some(Vec2::new(640.0, 480.0));
	native_options.initial_window_size = Some(Vec2::new(640.0, 480.0));

	eframe::run_native(
		"dslr-trichrome",
		native_options,
		Box::new(|_cc| Box::new(DslrTrichrome::new())),
	)
	.unwrap();

	std::process::exit(0)
}

struct SelectedImage {
	filename: Option<String>,
	working_thread: Option<JoinHandle<Result<Image<u8, Srgb>, rawproc::Error>>>,
	data: Option<Image<u8, Srgb>>,
	file_is_error: bool,
}

impl SelectedImage {
	pub fn new() -> Self {
		SelectedImage {
			filename: None,
			working_thread: None,
			data: None,
			file_is_error: false,
		}
	}

	pub fn new_selection(&mut self, fname: String) {
		self.filename = Some(fname.clone());
		self.working_thread = Some(Self::spawn_work_thread(fname));
		self.file_is_error = false;
	}

	pub fn working(&self) -> bool {
		self.working_thread.is_some()
	}

	pub fn work_ready(&self) -> bool {
		self.working_thread
			.as_ref()
			.map(|hndl| hndl.is_finished())
			.unwrap_or(false)
	}

	pub fn maybe_finish_work(&mut self) -> Option<Result<&Image<u8, Srgb>, rawproc::Error>> {
		if self.work_ready() {
			match self.working_thread.take() {
				None => None,
				Some(hndl) => match hndl.join().unwrap() {
					Err(e) => {
						self.file_is_error = true;
						Some(Err(e))
					}
					Ok(img) => {
						self.file_is_error = false;
						self.data = Some(img);
						Some(Ok(self.data.as_ref().unwrap()))
					}
				},
			}
		} else {
			None
		}
	}

	fn spawn_work_thread(fname: String) -> JoinHandle<Result<Image<u8, Srgb>, rawproc::Error>> {
		std::thread::spawn(|| {
			let mut file = File::open(fname).unwrap();
			rawproc::decode(&mut file).map(|mut img| {
				img.crop();
				img.whitebalance();
				img.debayer().to_xyz().to_linsrgb().gamma().floats().bytes()
			})
		})
	}
}

struct DslrTrichrome {
	image: Option<ColorImage>,
	red: SelectedImage,

	texture: Option<TextureHandle>,
}

impl eframe::App for DslrTrichrome {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		if self.red.working() {
			match self.red.maybe_finish_work() {
				None | Some(Err(_)) => (),
				Some(Ok(img)) => match self.image.as_mut() {
					None => (),
					Some(clr) => {
						let mut dest = vec![0; clr.width() * clr.height() * 3];

						let start = Instant::now();
						resize::new(
							img.width,
							img.height,
							clr.width(),
							clr.height(),
							resize::Pixel::RGB8,
							resize::Type::Triangle,
						)
						.unwrap()
						.resize(img.data.as_rgb(), dest.as_rgb_mut())
						.unwrap();
						println!("Resize took {}ms", start.elapsed().as_millis());

						for (idx, pix) in clr.pixels.iter_mut().enumerate() {
							*pix = Color32::from_rgb(dest[idx * 3], pix.g(), pix.b())
						}

						self.texture
							.as_mut()
							.unwrap()
							.set(clr.clone(), Default::default());
					}
				},
			}
		}

		let img = self.image.as_ref().unwrap().clone();
		let texture: &TextureHandle = self
			.texture
			.get_or_insert_with(|| ctx.load_texture("image", img, Default::default()));

		egui::CentralPanel::default().show(ctx, |ui| {
			let avsize = ui.available_size();
			let mut tsize = texture.size_vec2();

			let thdivw = tsize.y / tsize.x;
			let twdivh = tsize.x / tsize.y;

			tsize.y = (avsize.x * thdivw).min(2.0 * avsize.y / 3.0);
			tsize.x = tsize.y * twdivh;

			//println!("{tsize:?}");

			ui.vertical(|ui| {
				ui.horizontal(|ui| {
					ui.with_layout(
						egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
						|ui| ui.image(texture, tsize),
					)
				});

				ui.horizontal(|ui| {
					ui.label("Manipulating Channel: ");
					ui.label(RichText::new("none").color(Color32::LIGHT_GRAY));
				});

				let clr_r = Color32::from_rgb(255, 64, 64);
				let clr_g = Color32::from_rgb(64, 255, 64);
				let clr_b = Color32::from_rgb(64, 64, 255);

				ui.horizontal(|ui| {
					ui.button(RichText::new("Red").color(clr_r));
					ui.button(RichText::new("Green").color(clr_g));
					ui.button(RichText::new("Blue").color(clr_b));
				});

				ui.allocate_space(Vec2::new(1.0, 8.0));

				ui.label("Image Selection");
				ui.horizontal(|ui| {
					if ui.button(RichText::new("Red").color(clr_r)).clicked() {
						if let Some(path) = rfd::FileDialog::new().pick_file() {
							self.red.new_selection(path.to_string_lossy().into_owned());
						}
					}

					ui.label(self.red.filename.as_deref().unwrap_or("No file selected"));
				});
				ui.horizontal(|ui| {
					ui.button(RichText::new("Green").color(clr_g));
					ui.label("Filename");
				});
				ui.horizontal(|ui| {
					ui.button(RichText::new("Blue").color(clr_b));
					ui.label("Filename");
				});

				ui.allocate_space(ui.available_size());
			});
		});
	}
}

impl DslrTrichrome {
	pub fn new() -> Self {
		let img = ColorImage::from_rgb([1000, 666], &[0u8; 1000 * 666 * 3]);

		Self {
			image: Some(img),
			red: SelectedImage::new(),

			texture: None,
		}
	}
}
