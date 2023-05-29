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

struct SelectedChannel {
	channel: Channel,
	filename: Option<String>,
	file_is_error: bool,
	working_thread: Option<JoinHandle<Result<Image<u8, Srgb>, rawproc::Error>>>,

	data: Option<Vec<u8>>,
	width: usize,
	height: usize,
}

impl SelectedChannel {
	pub fn new(channel: Channel) -> Self {
		Self {
			channel,
			filename: None,
			working_thread: None,
			data: None,
			width: 0,
			height: 0,
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

	pub fn maybe_finish_work(&mut self) -> Option<Result<BorrowImage, rawproc::Error>> {
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

						self.width = img.width;
						self.height = img.height;
						self.data = Some(Self::extract_channe(img, self.channel));

						Some(Ok(BorrowImage {
							data: self.data.as_ref().unwrap(),
							width: self.width,
							height: self.height,
						}))
					}
				},
			}
		} else {
			None
		}
	}

	fn extract_channe(img: Image<u8, Srgb>, channel: Channel) -> Vec<u8> {
		let chidx = channel.index();

		let Image {
			width,
			height,
			mut data,
			..
		} = img;

		for idx in 0..width * height {
			data[idx] = data[idx * 3 + chidx];
		}

		data.resize(width * height, 0);
		data
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

struct BorrowImage<'a> {
	data: &'a [u8],
	width: usize,
	height: usize,
}

#[derive(Clone, Copy, Debug)]
enum Channel {
	Red,
	Green,
	Blue,
}

impl Channel {
	pub fn index(&self) -> usize {
		match self {
			Self::Red => 0,
			Self::Green => 1,
			Self::Blue => 2,
		}
	}
}

struct DslrTrichrome {
	image: Option<ColorImage>,
	red: SelectedChannel,
	green: SelectedChannel,
	blue: SelectedChannel,

	texture: Option<TextureHandle>,
}

impl eframe::App for DslrTrichrome {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		self.poll_channel_work(Channel::Red);
		self.poll_channel_work(Channel::Green);
		self.poll_channel_work(Channel::Blue);

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

				macro_rules! image_selection {
					($button:literal, $color:expr, $selectedchannel:expr) => {
						ui.horizontal(|ui| {
							if ui.button(RichText::new($button).color($color)).clicked() {
								if let Some(path) = rfd::FileDialog::new().pick_file() {
									$selectedchannel
										.new_selection(path.to_string_lossy().into_owned());
								}
							}

							ui.label(
								$selectedchannel
									.filename
									.as_deref()
									.unwrap_or("No file selected"),
							);
						});
					};
				}

				ui.label("Image Selection");
				image_selection!("Red", clr_r, self.red);
				image_selection!("Green", clr_g, self.green);
				image_selection!("Blue", clr_b, self.blue);

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
			red: SelectedChannel::new(Channel::Red),
			green: SelectedChannel::new(Channel::Green),
			blue: SelectedChannel::new(Channel::Blue),

			texture: None,
		}
	}

	pub fn poll_channel_work(&mut self, channel: Channel) {
		let selected = match channel {
			Channel::Red => &mut self.red,
			Channel::Green => &mut self.green,
			Channel::Blue => &mut self.blue,
		};

		if selected.working() {
			match selected.maybe_finish_work() {
				None | Some(Err(_)) => (),
				Some(Ok(img)) => match self.image.as_mut() {
					None => (),
					Some(clr) => {
						let mut dest = vec![0; clr.width() * clr.height()];

						let start = Instant::now();
						resize::new(
							img.width,
							img.height,
							clr.width(),
							clr.height(),
							resize::Pixel::Gray8,
							resize::Type::Triangle,
						)
						.unwrap()
						.resize(img.data.as_gray(), dest.as_gray_mut())
						.unwrap();
						println!("Resize took {}ms", start.elapsed().as_millis());

						for (idx, pix) in clr.as_raw_mut().chunks_mut(4).enumerate() {
							pix[channel.index()] = dest[idx];
						}

						self.texture
							.as_mut()
							.unwrap()
							.set(clr.clone(), Default::default());
					}
				},
			}
		}
	}
}
