use std::{
    io::Write,
    mem::size_of,
    path::PathBuf,
    sync::mpsc::{channel, Sender, TryRecvError},
};

use give::Give;
use rawproc::{
    debayer::{Debayer, Interpolation},
    image::{Image, RgbImage, SensorImage},
};

use crate::{cli::CliArgs, subsample};

pub struct Tui {
    file_path: PathBuf,
    image: EditingImage,
    preview: Option<Preview>,
}

impl Tui {
    pub fn new(cliargs: CliArgs) -> Tui {
        if cliargs.in_is_dir {
            panic!("Only single images currently, sorry");
        }

        let file_path = cliargs.in_path;
        let sensor = rawproc::read_file(file_path.to_str().unwrap());
        // This only works when we subsample, so uh, UH
        let sensor = subsample::subsample(sensor);

        let image = EditingImage::builder(sensor).build();

        Tui {
            file_path,
            image,
            preview: None,
        }
    }

    pub fn handoff(mut self) -> ! {
        let (tx, rx) = channel();

        println!("Image cache at: {}MB", self.image.megabytes());

        let cmd = CommandLine { tx };
        std::thread::spawn(|| cmd.run());

        loop {
            match rx.try_recv() {
                Ok(line) => self.process_command(line),
                Err(TryRecvError::Disconnected) => std::process::exit(0),
                Err(TryRecvError::Empty) => (),
            };

            if let Some(ref mut window) = self.preview {
                window.give.window_events();
            }
        }
    }

    fn process_command(&mut self, line: String) {
        let mut splits = line.trim().split(' ');
        let command = splits.next();

        match command {
            None => eprintln!("No command!"),
            Some("preview") => match self.preview {
                Some(_) => {
                    eprintln!("Preview window already open!")
                }
                None => {
                    print!("Building preview window...");
                    std::io::stdout().flush().unwrap();
                    self.preview = Some(Preview::new(&self.image.done));
                    println!("Done!");
                }
            },
            Some("close_preview") => match self.preview.take() {
                None => eprintln!("No preview open!"),
                Some(_) => println!("Closed preview!"),
            },
            Some("exposure") => match splits.next().map(|s| s.parse()) {
                None | Some(Err(_)) => {
                    eprintln!("usage: exposure <ev_value>\nexample:\n\texposure 1.3")
                }
                Some(Ok(ev)) => {
                    self.image.exposure(Some(ev));
                    self.image_changed()
                }
            },
            Some(cmd) => eprintln!("Unrecognized command '{cmd}'"),
        }
    }

    fn image_changed(&mut self) {
        if let Some(ref mut prev) = self.preview {
            prev.update(&self.image.done);
        }
    }
}

struct CommandLine {
    tx: Sender<String>,
}

impl CommandLine {
    pub fn run(self) -> ! {
        loop {
            let mut buf = String::new();
            print!("> ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut buf).unwrap();
            print!("{buf}");
            std::io::stdout().flush().unwrap();
            self.tx.send(buf).unwrap();
        }
    }
}

struct Preview {
    give: Give,
}

impl Preview {
    pub fn new(image: &RgbImage<u8>) -> Self {
        let mut this = Self { give: Give::new() };
        this.update(image);
        this.give.make_window(640, 480);
        this
    }

    pub fn update(&mut self, image: &RgbImage<u8>) {
        self.give.display_buffered_rgb8(
            image.meta.width as usize,
            image.meta.height as usize,
            image.data.clone(),
        );
    }
}

struct EditingImage {
    raw: SensorImage<u16>,

    black_levels: Option<Color<u16>>,
    white_balance: Option<Color<f32>>,
    exposure: Option<f32>,

    /// The image after:
    /// - black level correction
    /// - white balancing
    /// - exposure
    adjusted: SensorImage<f32>,

    /// The adjusted_sensor image with these, too:
    /// - Debayering
    /// - sRGB conversion
    srgb: RgbImage<f32>,

    brightness: Option<f32>,
    saturation: Option<f32>,
    contrast: Option<f32>,

    /// The final image after all adjustments are applied
    done: RgbImage<u8>,
}

impl EditingImage {
    pub fn builder(image: SensorImage<u16>) -> EditingImageBuilder {
        EditingImageBuilder::new(image)
    }

    pub fn major_size(&self) -> usize {
        let raw = self.raw.data().len() * size_of::<u16>();
        let adjusted = self.adjusted.data().len() * size_of::<f32>();
        let srgb = self.srgb.data().len() * size_of::<f32>();
        let done = self.done.data().len() * size_of::<u8>();

        raw + adjusted + srgb + done
    }

    pub fn kilobytes(&self) -> usize {
        self.major_size() / 1024
    }

    pub fn megabytes(&self) -> usize {
        self.kilobytes() / 1024
    }

    pub fn exposure(&mut self, exposure: Option<f32>) {
        self.exposure = exposure;
        self.step1();
    }

    /// Run step1 and everything above
    fn step1(&mut self) {
        let adjusted = step1(
            self.raw.clone(),
            self.black_levels,
            self.white_balance,
            self.exposure,
        );
        let srgb = step2(adjusted.clone());
        let done = step3(
            srgb.clone(),
            self.brightness,
            self.saturation,
            self.contrast,
        );

        self.adjusted = adjusted;
        self.srgb = srgb;
        self.done = done;
    }
}

struct EditingImageBuilder {
    image: SensorImage<u16>,
    black_levels: Option<Color<u16>>,
    white_balance: Option<Color<f32>>,
    exposure: Option<f32>,
    brightness: Option<f32>,
    saturation: Option<f32>,
    contrast: Option<f32>,
}

impl EditingImageBuilder {
    pub fn new(image: SensorImage<u16>) -> Self {
        Self {
            image,
            black_levels: None,
            white_balance: None,
            exposure: None,
            brightness: None,
            saturation: None,
            contrast: None,
        }
    }

    pub fn build(self) -> EditingImage {
        let EditingImageBuilder {
            image,
            black_levels,
            white_balance,
            exposure,
            brightness,
            saturation,
            contrast,
        } = self;

        let adjusted = step1(image.clone(), black_levels, white_balance, exposure);
        let srgb = step2(adjusted.clone());
        let done = step3(srgb.clone(), brightness, saturation, contrast);

        EditingImage {
            raw: image,
            black_levels,
            white_balance,
            exposure,
            adjusted,
            srgb,
            brightness,
            saturation,
            contrast,
            done,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Color<T: Copy> {
    r: T,
    g: T,
    b: T,
}

impl<T: Copy> Into<(T, T, T)> for Color<T> {
    fn into(self) -> (T, T, T) {
        (self.r, self.g, self.b)
    }
}

fn step1(
    mut sensor: SensorImage<u16>,
    black_levels: Option<Color<u16>>,
    white_balance: Option<Color<f32>>,
    exposure: Option<f32>,
) -> SensorImage<f32> {
    sensor.black_levels(black_levels.map(|bl| bl.into()));
    let mut floats = sensor.into_floats();
    floats.white_balance(white_balance.map(|wb| wb.into()));
    if let Some(ev) = exposure {
        floats.exposure(ev);
    }
    floats
}

fn step2(adjusted: SensorImage<f32>) -> RgbImage<f32> {
    let mut debayered = Debayer::new(adjusted).interpolate(Interpolation::Bilinear);
    debayered.to_srgb();
    debayered
}

fn step3(
    srgb: RgbImage<f32>,
    brightness: Option<f32>,
    saturation: Option<f32>,
    contrast: Option<f32>,
) -> RgbImage<u8> {
    let mut hsv = srgb.into_hsv();

    if let Some(v) = brightness {
        hsv.brightness(v);
    }

    if let Some(s) = saturation {
        hsv.saturation(s);
    }

    let mut floats = hsv.into_rgb();

    if let Some(c) = contrast {
        floats.contrast(c);
    }

    floats.into_u8s()
}
