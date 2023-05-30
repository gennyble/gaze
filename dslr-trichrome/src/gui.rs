use core::fmt;
use std::{cell::Cell, fs::File, thread::JoinHandle, time::Instant};

use camino::Utf8PathBuf;
use egui::{Color32, ColorImage, Layout, RichText, TextureHandle, Vec2};
use egui_dock::Tree;
use rawproc::{colorspace::Srgb, image::Image};
use rgb::FromSlice;

use crate::TrichromedImage;

/*
Hey gen! Briefly, some words.

We'd like it if the control tab thing was anchored to the bottom and the image
filled the rest of the height in the window (within aspect ratio bounds).

It would also be nice if the UI could change so the image is to the left of the
controls if the image is vertical. Perhaps instead of automatically we could put
a toggle in a...

settings tab. it should have a save field as well as what to save. i'd like the
ability to output multiple sizes if i want, for publishing i guess, in a few
different formats. maybe that's a separate program where we read a PNG and do
that. that could be useful. a CLI, too, and a library? I guess we made that with
imgout before, but then you rolled back the code! gosh. can you save the settings,
too, in like XDG_CONFIG/dslr-trichrome. or whatever it is if we rename it. perhaps
this is what gaze can be. it seems like it might be easy to be like "who cares if the
input is one image or trichrome". maybe it *does* end up all getting folded together.
would certainly make the debayering easier. right!

debayering. it's currently the bad style. the idea was to use rawproc and let you
pass in separate buffers for the channels. and then you would just pass in the same
buffer three times to do it normally. we could extract that away. extract? abstract.

oh, hey! don't get distracte by the UI anymore? it works. just focus on the offset.
it'll be great! i promise. you can make it all okay later. focus on the main purpose
of the thing. please? thanks.

tired, so it's time to sleep. you can do it! <3
*/

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
						self.data = Some(Self::extract_channel(img, self.channel));

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

	fn extract_channel(img: Image<u8, Srgb>, channel: Channel) -> Vec<u8> {
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

	pub fn color_text(&self, text: impl Into<String>) -> RichText {
		RichText::new(text).color(self.color32())
	}

	pub fn name_label(&self) -> RichText {
		self.color_text(self.to_string())
	}

	pub fn color32(&self) -> Color32 {
		match self {
			Self::Red => RED,
			Self::Green => GREEN,
			Self::Blue => BLUE,
		}
	}
}

impl fmt::Display for Channel {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Channel::Red => write!(f, "Red"),
			Channel::Green => write!(f, "Green"),
			Channel::Blue => write!(f, "Blue"),
		}
	}
}

const RED: Color32 = Color32::from_rgb(255, 64, 64);
const GREEN: Color32 = Color32::from_rgb(64, 255, 64);
const BLUE: Color32 = Color32::from_rgb(64, 64, 255);

enum Tab {
	Input,
	Offset,
	Output,
}

impl Tab {
	pub fn str(&self) -> &'static str {
		match self {
			Tab::Input => "Input",
			Tab::Offset => "Offset",
			Tab::Output => "Output",
		}
	}
}

impl fmt::Display for Tab {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.str())
	}
}

struct DslrTrichrome {
	image: Option<ColorImage>,

	red: SelectedChannel,
	green: SelectedChannel,
	blue: SelectedChannel,

	selected: Channel,
	red_offset: (isize, isize),
	green_offset: (isize, isize),
	blue_offset: (isize, isize),

	texture: Option<TextureHandle>,
	// This is an option so we can take() it, use DslrTrichrome as the TabViewer, and then put it back.
	tabs: Option<Tree<Tab>>,
}

impl eframe::App for DslrTrichrome {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		self.poll_channel_work(Channel::Red);
		self.poll_channel_work(Channel::Green);
		self.poll_channel_work(Channel::Blue);

		egui::CentralPanel::default().show(ctx, |ui| {
			let mut reoffset = false;
			ui.input(|i| {
				let offset = match self.selected {
					Channel::Red => &mut self.red_offset,
					Channel::Green => &mut self.green_offset,
					Channel::Blue => &mut self.blue_offset,
				};

				let on = 10;
				if i.key_pressed(egui::Key::ArrowUp) {
					offset.1 -= on;
					reoffset = true;
				} else if i.key_pressed(egui::Key::ArrowDown) {
					offset.1 += on;
					reoffset = true;
				}

				if i.key_pressed(egui::Key::ArrowLeft) {
					offset.0 -= on;
					reoffset = true;
				} else if i.key_pressed(egui::Key::ArrowRight) {
					offset.0 += on;
					reoffset = true;
				}
			});

			if reoffset {
				self.redraw_selected_channel();
				self.make_texture();
			}

			ui.vertical(|ui| {
				let avsize = ui.available_size();
				ui.allocate_ui(Vec2::new(avsize.x, avsize.y / 2.0), |ui| {
					ui.horizontal(|ui| {
						let img = self.image.as_ref().unwrap().clone();
						let texture: &TextureHandle = self.texture.get_or_insert_with(|| {
							ctx.load_texture("image", img, Default::default())
						});

						let mut tsize = texture.size_vec2();

						let thdivw = tsize.y / tsize.x;
						let twdivh = tsize.x / tsize.y;

						tsize.y = (avsize.x * thdivw).min(2.0 * avsize.y / 3.0);
						tsize.x = tsize.y * twdivh;

						ui.with_layout(
							egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
							|ui| ui.image(texture, tsize),
						)
					});
				});

				let mut tree = self
					.tabs
					.take()
					.expect("There was no tab tree. this should be impossible!");

				egui_dock::DockArea::new(&mut tree)
					.show_close_buttons(false)
					.style(egui_dock::Style::from_egui(ui.style().as_ref()))
					.show_inside(ui, self);

				self.tabs = Some(tree);
			});
		});
	}
}

impl egui_dock::TabViewer for DslrTrichrome {
	type Tab = Tab;

	fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
		match tab {
			Tab::Offset => self.ui_offset_tab(ui),
			Tab::Input => self.ui_input_tab(ui),
			Tab::Output => self.ui_output_tab(ui),
		}
	}

	fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
		tab.str().into()
	}
}

impl DslrTrichrome {
	/// The largest side of the preview.
	const PREVIEW_LARGE: f32 = 1000.0;

	pub fn new() -> Self {
		//TODO: gen- We should use the PREVIEW_LARGE here. to make a 4:3 image
		let img = ColorImage::from_rgb([1000, 666], &[0u8; 1000 * 666 * 3]);

		let tree = Tree::new(vec![Tab::Input, Tab::Offset, Tab::Output]);

		Self {
			image: Some(img),

			red: SelectedChannel::new(Channel::Red),
			green: SelectedChannel::new(Channel::Green),
			blue: SelectedChannel::new(Channel::Blue),

			selected: Channel::Red,
			red_offset: (0, 0),
			green_offset: (0, 0),
			blue_offset: (0, 0),

			texture: None,
			tabs: Some(tree),
		}
	}

	//TODO: gen- Check if the image needs resizing or aspect change. Better yet, break out the blitting (?)
	//to another function so we can redraw on offset change, too
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
						if clr.width() < img.width || clr.height() < img.height {
							*clr = ColorImage::new(
								[clr.width().max(img.width), clr.height().max(img.height)],
								Color32::BLACK,
							);
						}

						self.redraw_channel(channel);
						self.make_texture();
					}
				},
			}
		}
	}

	pub fn make_texture(&mut self) {
		let prev_width = Self::PREVIEW_LARGE as usize;
		let prev_height = 666;

		if let Some(img) = self.image.as_ref() {
			let mut dest = vec![0; prev_width as usize * prev_height * 3];

			let start = Instant::now();
			resize::new(
				img.width(),
				img.height(),
				prev_width,
				prev_height,
				resize::Pixel::RGB8,
				resize::Type::Triangle,
			)
			.unwrap()
			.resize(img.as_raw().as_rgb(), dest.as_rgb_mut())
			.unwrap();
			println!("Resize took {}ms", start.elapsed().as_millis());

			let colorimage = ColorImage::from_rgb([prev_width, prev_height], &dest);

			self.texture
				.as_mut()
				.unwrap()
				.set(colorimage, Default::default());
		}
	}

	fn redraw_selected_channel(&mut self) {
		self.redraw_channel(self.selected)
	}

	fn redraw_channel(&mut self, channel: Channel) {
		let (offset, channel_img, width, height) = match channel {
			Channel::Red => match self.red.data.as_ref() {
				None => return,
				Some(data) => (self.red_offset, data, self.red.width, self.red.height),
			},
			Channel::Green => match self.green.data.as_ref() {
				None => return,
				Some(data) => (self.green_offset, data, self.green.width, self.green.height),
			},
			Channel::Blue => match self.blue.data.as_ref() {
				None => return,
				Some(data) => (self.blue_offset, data, self.blue.width, self.blue.height),
			},
		};

		let img = if let Some(img) = self.image.as_mut() {
			img
		} else {
			return;
		};

		'rows: for y in 0..height {
			'cols: for x in 0..width {
				let xoff = x as isize + offset.0;
				let yoff = y as isize + offset.1;

				if yoff < 0 {
					continue 'rows;
				} else if yoff >= img.height() as isize {
					return;
				}

				if xoff < 0 {
					continue 'cols;
				} else if xoff >= img.width() as isize {
					continue 'rows;
				}

				let channel_idx = y as usize * width + x as usize;
				let idx = yoff as usize * img.width() + xoff as usize;
				img.as_raw_mut()[idx * 3 + channel.index()] = channel_img[channel_idx];
			}
		}
	}

	fn ui_offset_tab(&mut self, ui: &mut egui::Ui) {
		ui.label("Move the selected channel with the arrow keys");
		ui.horizontal(|ui| {
			ui.label("Manipulating Channel: ");
			ui.label(self.selected.name_label());
		});

		macro_rules! offset_button {
			($ui:expr, $channel:expr) => {
				let offset = match $channel {
					Channel::Red => self.red_offset,
					Channel::Green => self.green_offset,
					Channel::Blue => self.blue_offset,
				};

				$ui.vertical(|ui| {
					if ui.button($channel.name_label()).clicked() {
						self.selected = $channel;
					}

					ui.label($channel.color_text(format!("{} / {}", offset.0, offset.1)));
				})
			};
		}

		ui.horizontal(|ui| {
			offset_button!(ui, Channel::Red);
			offset_button!(ui, Channel::Green);
			offset_button!(ui, Channel::Blue);
		});
	}

	fn ui_input_tab(&mut self, ui: &mut egui::Ui) {
		macro_rules! image_selection {
			($selectedchannel:expr) => {
				ui.horizontal(|ui| {
					if ui.button($selectedchannel.channel.name_label()).clicked() {
						if let Some(path) = rfd::FileDialog::new().pick_file() {
							$selectedchannel.new_selection(path.to_string_lossy().into_owned());
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
		image_selection!(self.red);
		image_selection!(self.green);
		image_selection!(self.blue);

		ui.allocate_space(ui.available_size());
	}

	fn ui_output_tab(&mut self, ui: &mut egui::Ui) {
		if ui.button("Save PNG").clicked() {
			if let Some(path) = rfd::FileDialog::new().save_file() {
				match self.image.as_ref() {
					None => {
						eprintln!("No image to save!");
					}
					Some(img) => {
						println!("PNG Output: {}x{}", img.width(), img.height());
						println!(
							"{}px - sqrt {} - divw {}",
							img.pixels.len(),
							(img.pixels.len() as f32).sqrt() as usize,
							img.pixels.len() / img.width()
						);
						println!("{} / px", img.as_raw().len() / img.pixels.len());

						// Un A
						let una_start = Instant::now();
						let mut data = img.as_raw().to_vec();
						for idx in 0..img.width() * img.height() {
							data[idx * 3] = data[idx * 4];
							data[idx * 3 + 1] = data[idx * 4 + 1];
							data[idx * 3 + 2] = data[idx * 4 + 2];
						}
						data.resize(img.width() * img.height() * 3, 0);
						println!("De-alpha took {}ms", una_start.elapsed().as_millis());

						let trimg = TrichromedImage {
							width: img.width(),
							height: img.height(),
							data,
						};
						trimg.png(Utf8PathBuf::try_from(path).unwrap())
					}
				}
			}
		}
	}
}
