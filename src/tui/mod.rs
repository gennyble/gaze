use std::{
    io::Write,
    mem::size_of,
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    time::Duration,
};

use colorogram::make_histogram;
use give::{Give, GiveWindowBuilder, WindowId};
use rawproc::{
    debayer::{Debayer, Interpolation},
    image::{Image, RgbImage, SensorImage},
};

use crate::{cli::CliArgs, subsample};

pub struct Tui {
    file_path: PathBuf,
    image: EditingImage,

    give: Give,
    preview: Option<Preview>,
    histogram: Option<Preview>,

    last_command: Option<String>,
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
            give: Give::new(),
            preview: None,
            histogram: None,
            last_command: None,
        }
    }

    pub fn handoff(mut self) -> ! {
        let (command_tx, command_rx) = channel();
        let (stdout_tx, stdout_rx) = channel();

        println!("Image cache at: {}MB", self.image.megabytes());

        let cmd = CommandLine {
            tx: command_tx,
            rx: stdout_rx,
        };
        std::thread::spawn(|| cmd.run());

        loop {
            match command_rx.try_recv() {
                Ok(line) => self.process_command(line, &stdout_tx),
                Err(TryRecvError::Disconnected) => std::process::exit(0),
                Err(TryRecvError::Empty) => (),
            };

            if self.preview.is_some() || self.histogram.is_some() {
                self.give.window_events();
            }
        }
    }

    fn process_command(&mut self, line: String, tx: &Sender<Message>) {
        let mut splits = line.trim().split(' ');
        let command = splits.next();
        //let arguments = splits.collect();

        macro_rules! msg {
            ($($arg:tt)*) => {
                tx.send(Message::Normal(std::fmt::format(format_args!($($arg)*)))).unwrap()
            };
        }

        macro_rules! err {
            ($($arg:tt)*) => {
                tx.send(Message::Error(std::fmt::format(format_args!($($arg)*)))).unwrap()
            };
        }

        match command {
            None => eprintln!("No command!"),
            Some("preview") => match self.preview {
                Some(_) => {
                    err!("Preview window already open!")
                }
                None => {
                    msg!("Opening preview...");
                    self.preview = Some(Preview::new(&mut self.give, self.image.done.clone()));
                    msg!("Done!");
                }
            },
            Some("close_preview") => match self.preview.take() {
                None => err!("No preview open!"),
                Some(_) => msg!("Closed preview!"),
            },
            Some("histogram") => match self.histogram {
                Some(_) => {
                    err!("Histogram window already open!")
                }
                None => {
                    msg!("Opening Histogram...");
                    let himg = self.make_histogram_image();
                    self.histogram = Some(Preview::new(&mut self.give, himg));
                    msg!("Done!");
                }
            },
            Some("exposure") => match splits.next().map(|s| s.parse()) {
                None | Some(Err(_)) => {
                    msg!("usage: exposure <ev_value>\nexample:\n\texposure 1.3")
                }
                Some(Ok(ev)) => {
                    self.image.exposure(Some(ev));
                    self.image_changed()
                }
            },
            Some("goodbye") => std::process::exit(0),
            Some(cmd) => err!("Unrecognized command '{cmd}'"),
        }
    }

    fn make_histogram_image(&self) -> Image8 {
        let histo = make_histogram(self.image.done.data(), 640, 480);
        Image8 {
            data: histo,
            width: 640,
            height: 480,
        }
    }

    fn image_changed(&mut self) {
        if let Some(ref mut prev) = self.preview {
            prev.update(&mut self.give, self.image.done.clone());
        }

        if self.histogram.is_some() {
            let himg = self.make_histogram_image();
            if let Some(ref mut histo) = self.histogram {
                histo.update(&mut self.give, himg)
            }
        }
    }
}

trait Command {
    fn usage() -> &'static str;
    fn run(image: &mut EditingImage, arguments: Vec<String>) -> Result<(), CommandError>;
}

enum CommandError {
    MissingArgument { index: usize, argument_name: String },
    InvalidArgument { argument: String, reason: String },
}

impl CommandError {
    pub fn missing_argument<S: Into<String>>(idx: usize, argument_name: S) -> Self {
        CommandError::MissingArgument {
            index: idx,
            argument_name: argument_name.into(),
        }
    }
}

struct Exposure;
impl Command for Exposure {
    fn usage() -> &'static str {
        "exposure clear OR exposure <ev>"
    }

    fn run(image: &mut EditingImage, arguments: Vec<String>) -> Result<(), CommandError> {
        /*let ev = match arguments.get(0) {
            None => return Err(CommandError::missing_argument(0, "ev")),
            Some(ev) => match
        }*/
        todo!()
    }
}

struct CommandLine {
    tx: Sender<String>,
    rx: Receiver<Message>,
}

impl CommandLine {
    const PROMPT: &'static str = "> ";

    pub fn run(self) -> ! {
        let (stdin_tx, stdin_rx) = channel();
        let _join = Self::do_stdin(stdin_tx);

        print!("{}", Self::PROMPT);
        std::io::stdout().flush().unwrap();

        loop {
            match stdin_rx.try_recv() {
                Ok(line) => {
                    if let Err(e) = self.tx.send(line) {
                        eprintln!("Failed to send stdin back to main thread! {e}");
                    }
                    print!("{}", Self::PROMPT);
                    std::io::stdout().flush().unwrap();
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    eprintln!("stdin channel closed! Did the thread die?")
                }
            }

            match self.rx.try_recv() {
                Ok(Message::Normal(line)) => {
                    print!("\r{line}\n{}", Self::PROMPT);
                    std::io::stdout().flush().unwrap();
                }
                Ok(Message::Error(error)) => {
                    print!("\r{error}\n{}", Self::PROMPT);
                    std::io::stdout().flush().unwrap();
                }
                Err(TryRecvError::Disconnected) => {
                    eprintln!("log channel closed! What happened?")
                }
                Err(TryRecvError::Empty) => std::thread::sleep(Duration::from_micros(100)),
            }
        }
    }

    fn do_stdin(tx: Sender<String>) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || loop {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).unwrap();

            match tx.send(buf) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("stdin transmit channel broke! {e}");
                    break;
                }
            }
        })
    }
}

enum Message {
    Error(String),
    Normal(String),
}

struct Image8 {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

impl From<RgbImage<u8>> for Image8 {
    fn from(value: RgbImage<u8>) -> Self {
        Self {
            data: value.data,
            width: value.meta.width as usize,
            height: value.meta.height as usize,
        }
    }
}

struct Preview {
    window: WindowId,
}

impl Preview {
    pub fn new<M: Into<Image8>>(give: &mut Give, image: M) -> Self {
        let image = image.into();
        let window = GiveWindowBuilder::new()
            .buffer_rgb8(image.data, image.width, image.height)
            .build(give);

        Self { window }
    }

    pub fn update<M: Into<Image8>>(&mut self, give: &mut Give, image: M) {
        let image = image.into();
        give.display_buffered_rgb8(
            &self.window,
            image.width as usize,
            image.height as usize,
            image.data,
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
