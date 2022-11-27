use std::{
    io::{stdout, Write},
    sync::mpsc::{Receiver, Sender, TryRecvError},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{poll, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::Print,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use super::{
    operation::{Action, Bounds, Draw, Magnitude, MenuItem, Operation, Toggle, Valuef32},
    Color,
};

#[derive(Clone, Debug)]
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

    selected: usize,
    operations: Vec<MenuItem>,
}

impl Interface {
    pub fn new(tx: Sender<OperationMessage>, rx: Receiver<Message>) -> Self {
        Self {
            tx,
            rx,

            width: 0,
            height: 0,
            cache: 0,

            selected: 0,
            operations: Self::make_operations(),
        }
    }

    fn make_operations() -> Vec<MenuItem> {
        let mut operations = vec![];

        let preview = Toggle::new("preview".into(), false, OperationMessage::TogglePreview);
        operations.push(MenuItem::Preview(preview));

        let histogram = Toggle::new("histogram".into(), false, OperationMessage::ToggleHistogram);
        operations.push(MenuItem::Histogram(histogram));

        let exposure = Valuef32::new("exposure", 0.0, |val| OperationMessage::Exposure(val));
        operations.push(MenuItem::Exposure(exposure));

        let brightness = Valuef32::new("brightness", 0.0, |val| OperationMessage::Brightness(val));
        operations.push(MenuItem::Brightness(brightness));

        let mut saturation =
            Valuef32::new("saturation", 1.0, |val| OperationMessage::Saturation(val));
        saturation.set_percent(true);
        operations.push(MenuItem::Saturation(saturation));

        let mut contrast = Valuef32::new("contrast", 1.0, |val| OperationMessage::Contrast(val));
        contrast.set_percent(true);
        operations.push(MenuItem::Contrast(contrast));

        operations
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

        let (width, height) = terminal::size()?;
        self.width = width;
        self.height = height;

        self.redraw().unwrap();
        self.draw_info(format!("{}x{}", width, height)).unwrap();

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
                self.operations.iter_mut().for_each(|mi| {
                    if let MenuItem::Preview(toggle) = mi {
                        toggle.set(flag);
                    }
                });

                self.draw_main()
            }
            Message::Histogram(flag) => {
                self.operations.iter_mut().for_each(|mi| {
                    if let MenuItem::Histogram(toggle) = mi {
                        toggle.set(flag);
                    }
                });

                self.draw_main()
            }
        }
    }

    fn event(&mut self) -> Result<bool, crossterm::ErrorKind> {
        match crossterm::event::read()? {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match code {
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.selected + 1 < self.operations.len() {
                        self.selected += 1;
                    }
                }
                KeyCode::Left => {
                    self.operation_message(Action::Decrease(Self::magnitude(modifiers)))
                }
                KeyCode::Right => {
                    self.operation_message(Action::Increase(Self::magnitude(modifiers)))
                }
                KeyCode::Enter => self.operation_message(Action::Enter),
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('d') => {
                    self.redraw()?;
                    self.draw_info(format!("Redrew UI!"))?;
                }
                KeyCode::Char('s') => self.tx.send(OperationMessage::Save).unwrap(),
                KeyCode::Char('p') => {
                    let messages = self
                        .operations
                        .iter_mut()
                        .filter_map(|mi| match mi {
                            MenuItem::Preview(prev) => {
                                prev.action(Action::Enter);
                                Some(prev.message())
                            }
                            MenuItem::Histogram(prev) => {
                                prev.action(Action::Enter);
                                Some(prev.message())
                            }
                            _ => None,
                        })
                        .collect::<Vec<OperationMessage>>();
                    messages.into_iter().for_each(|m| self.tx.send(m).unwrap());
                }
                _ => (),
            },
            Event::Resize(width, height) => {
                self.width = width;
                self.height = height;

                if width < 10 {
                    panic!("minimum width 10")
                }

                self.redraw()?;
            }
            _ => (),
        }

        self.draw_main()?;

        Ok(false)
    }

    fn magnitude(modifiers: KeyModifiers) -> Magnitude {
        if modifiers.contains(KeyModifiers::SHIFT) {
            Magnitude::Smaller
        } else if modifiers.contains(KeyModifiers::CONTROL) {
            Magnitude::Bigger
        } else {
            Magnitude::Normal
        }
    }

    fn operation_message(&mut self, action: Action) {
        let op = self.operations.get_mut(self.selected).unwrap();
        op.action(action);
        self.tx.send(op.message()).unwrap();
    }

    fn enter() -> Result<(), crossterm::ErrorKind> {
        execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        execute!(stdout(), cursor::Hide, cursor::MoveTo(0, 0))
    }

    fn redraw(&mut self) -> Result<(), crossterm::ErrorKind> {
        self.draw_header()?;
        self.draw_main()
    }

    fn draw_header(&self) -> Result<(), crossterm::ErrorKind> {
        let mut stdout = stdout();

        let cache = format!("IMG Cache {}MB", self.cache / (1024 * 1024));
        execute!(stdout, cursor::MoveTo(0, 0), Print(cache))
    }

    fn draw_info(&self, info: String) -> Result<(), crossterm::ErrorKind> {
        execute!(stdout(), cursor::MoveTo(0, 1), Print(info))
    }

    fn draw_main(&self) -> Result<(), crossterm::ErrorKind> {
        let start = 3;

        queue!(stdout(), cursor::MoveTo(0, start))?;

        let width = self.width.min(20) as usize;
        let bounds = Bounds {
            width,
            height: self.height as usize,
        };

        for op in &self.operations {
            let draw = op.draw(bounds.clone());

            match draw {
                Draw::Line(ln) => {
                    queue!(stdout(), Print(format!(" {ln}")), cursor::MoveToNextLine(1))?;
                }
                Draw::Box { lines } => {
                    todo!()
                }
            }
        }

        execute!(
            stdout(),
            cursor::MoveTo(0, start + self.selected as u16),
            Print(">")
        )
    }

    fn leave() -> Result<(), crossterm::ErrorKind> {
        disable_raw_mode()?;
        execute!(stdout(), cursor::Show, LeaveAlternateScreen)
    }
}
