use std::{
    io::{stdout, Write},
    sync::mpsc::{Receiver, Sender, TryRecvError},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{poll, Event, KeyCode, KeyEvent},
    execute, queue,
    style::Print,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use super::Color;

pub enum OperationMessage {
    BlackLevels(Color<u16>),
    WhiteBalance(Color<f32>),
    Exposure(f32),
    Brightness(f32),
    Saturation(f32),
    Contrast(f32),
    TogglePreview,
    ToggleHistogram,
    Shutdown,
    Save,
}

pub enum Message {
    Error(String),
    Normal(String),
    Cache(usize),
    Preview(bool),
    Histogram(bool),
}

pub struct Interface {
    tx: Sender<OperationMessage>,
    rx: Receiver<Message>,

    width: u16,
    height: u16,
    cache: usize,
    preview: bool,
    histogram: bool,

    selected: usize,
    black: u16,
    exposure: f32,
}

impl Interface {
    pub fn new(tx: Sender<OperationMessage>, rx: Receiver<Message>) -> Self {
        Self {
            tx,
            rx,

            width: 0,
            height: 0,
            cache: 0,
            preview: false,
            histogram: false,

            selected: 0,
            black: 0,
            exposure: 0.0,
        }
    }

    pub fn run(self) {
        match self.do_run() {
            Err(e) => {
                Self::leave().unwrap();
                eprintln!("failure: {e}");
                std::process::exit(-1);
            }
            Ok(_) => (),
        }
    }

    fn do_run(mut self) -> Result<(), crossterm::ErrorKind> {
        Self::enter()?;

        self.draw_header()?;
        self.draw_OperationMessages()?;

        let (width, height) = terminal::size()?;
        self.width = width;
        self.height = height;

        loop {
            match self.rx.try_recv() {
                Ok(msg) => self.process_message(msg)?,
                Err(TryRecvError::Disconnected) => {
                    eprintln!("log channel closed! What happened?")
                }
                Err(TryRecvError::Empty) => std::thread::sleep(Duration::from_micros(100)),
            }

            match poll(Duration::from_millis(100)) {
                Err(_) => {
                    self.tx.send(OperationMessage::Shutdown).unwrap();
                    break;
                }
                Ok(true) => {
                    if self.event()? {
                        self.tx.send(OperationMessage::Shutdown).unwrap();
                        break;
                    }
                }
                Ok(false) => continue,
            }
        }

        Self::leave()
    }

    fn process_message(&mut self, message: Message) -> Result<(), crossterm::ErrorKind> {
        match message {
            Message::Error(err) => self.draw_info(format!("error: {err}")),
            Message::Normal(message) => self.draw_info(message),
            Message::Cache(size) => {
                self.cache = size;
                self.draw_header()
            }
            Message::Preview(flag) => {
                self.preview = flag;
                self.draw_OperationMessages()
            }
            Message::Histogram(flag) => {
                self.histogram = flag;
                self.draw_OperationMessages()
            }
        }
    }

    fn event(&mut self) -> Result<bool, crossterm::ErrorKind> {
        match crossterm::event::read()? {
            Event::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.selected < 4 {
                        self.selected += 1;
                    }
                }
                KeyCode::Left => self.OperationMessage(false),
                KeyCode::Right => self.OperationMessage(true),
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('d') => {
                    self.redraw()?;
                    self.draw_info(format!("Redrew UI!"))?;
                }
                KeyCode::Char('s') => self.tx.send(OperationMessage::Save).unwrap(),
                _ => (),
            },
            Event::Resize(width, height) => {
                self.width = width;
                self.height = height;
                self.redraw()?;
            }
            _ => (),
        }

        self.draw_OperationMessages()?;

        Ok(false)
    }

    fn OperationMessage(&mut self, increase: bool) {
        match self.selected {
            0 => {
                self.tx.send(OperationMessage::TogglePreview).unwrap();
            }
            1 => {
                self.tx.send(OperationMessage::ToggleHistogram).unwrap();
            }
            2 => {
                if increase {
                    self.black += 1;
                } else {
                    if self.black > 0 {
                        self.black -= 1;
                    }
                }
                self.tx
                    .send(OperationMessage::BlackLevels(Color {
                        r: self.black,
                        g: self.black,
                        b: self.black,
                    }))
                    .unwrap();
            }
            3 => {
                let delta = if increase { 0.1 } else { -0.1 };
                self.exposure += delta;
                self.tx
                    .send(OperationMessage::Exposure(self.exposure))
                    .unwrap();
            }
            _ => (),
        }
    }

    fn enter() -> Result<(), crossterm::ErrorKind> {
        execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn redraw(&mut self) -> Result<(), crossterm::ErrorKind> {
        self.draw_header()?;
        self.draw_OperationMessages()
    }

    fn draw_header(&self) -> Result<(), crossterm::ErrorKind> {
        let mut stdout = stdout();

        let cache = format!("IMG Cache {}MB", self.cache / (1024 * 1024));
        queue!(stdout, cursor::MoveTo(0, 0), Print(cache))?;

        stdout.flush()
    }

    fn draw_info(&self, info: String) -> Result<(), crossterm::ErrorKind> {
        execute!(stdout(), cursor::MoveTo(0, 1), Print(info))
    }

    fn draw_OperationMessages(&self) -> Result<(), crossterm::ErrorKind> {
        let start = 3;

        execute!(
            stdout(),
            cursor::MoveTo(0, start),
            Print(format!(" preview  {}", Self::open_close(self.preview))),
            cursor::MoveToNextLine(1),
            Print(format!(" histogram  {}", Self::open_close(self.histogram))),
            cursor::MoveToNextLine(1),
            Print(format!(" black levels {}", self.black)),
            cursor::MoveToNextLine(1),
            Print(format!(" exposure {:.1}", self.exposure)),
            cursor::MoveTo(0, start + self.selected as u16),
            Print(">")
        )
    }

    fn open_close(flag: bool) -> &'static str {
        if flag {
            "open"
        } else {
            "closed"
        }
    }

    fn leave() -> Result<(), crossterm::ErrorKind> {
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)
    }
}
