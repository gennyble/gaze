use super::interface::OperationMessage;

pub trait Operation: Send {
    fn draw(&self, bounds: Bounds) -> Draw;
    fn message(&self) -> OperationMessage;
    fn action(&mut self, action: Action);
}

pub enum MenuItem {
    Preview(Toggle),
    Histogram(Toggle),
    Exposure(Valuef32),
    Brightness(Valuef32),
    Saturation(Valuef32),
    Contrast(Valuef32),
}

impl MenuItem {
    pub fn inner(&self) -> &dyn Operation {
        match self {
            MenuItem::Preview(toggle) => toggle,
            MenuItem::Histogram(toggle) => toggle,
            MenuItem::Exposure(ev) => ev,
            MenuItem::Brightness(b) => b,
            MenuItem::Saturation(s) => s,
            MenuItem::Contrast(c) => c,
        }
    }

    pub fn inner_mut(&mut self) -> &mut dyn Operation {
        match self {
            MenuItem::Preview(toggle) => toggle,
            MenuItem::Histogram(toggle) => toggle,
            MenuItem::Exposure(ev) => ev,
            MenuItem::Brightness(b) => b,
            MenuItem::Saturation(s) => s,
            MenuItem::Contrast(c) => c,
        }
    }
}

impl Operation for MenuItem {
    fn draw(&self, bounds: Bounds) -> Draw {
        self.inner().draw(bounds)
    }

    fn message(&self) -> OperationMessage {
        self.inner().message()
    }

    fn action(&mut self, action: Action) {
        self.inner_mut().action(action)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Action {
    Increase(Magnitude),
    Decrease(Magnitude),
    Enter,
}

#[derive(Copy, Clone, Debug)]
pub enum Magnitude {
    Bigger,
    Normal,
    Smaller,
}

pub enum Draw {
    Line(String),
    Box { lines: String },
}

#[derive(Clone, Debug)]
pub struct Bounds {
    pub width: usize,
    pub height: usize,
}

fn simple<S: Into<String>>(string: S, value_string: String, bounds: Bounds) -> Draw {
    let string = string.into();
    let max_len = string.len() + value_string.len() + 1;

    if max_len > bounds.width {
        let value_len = value_string.len() + 1;
        let drawn = format!("{}…{}", &string[..value_len], value_string);

        Draw::Line(drawn)
    } else {
        let width = bounds.width - value_string.len() - 1;
        let length_string = format!("{string:<width$} {value_string}");

        Draw::Line(length_string)
    }
}

pub struct Valuef32 {
    name: String,
    value: f32,
    operation_fn: Box<dyn Fn(f32) -> OperationMessage + Send>,
    percent: bool,
}

impl Valuef32 {
    pub fn new<S: Into<String>, F>(name: S, initial: f32, operation_fn: F) -> Self
    where
        F: Fn(f32) -> OperationMessage + Send + 'static,
    {
        Self {
            name: name.into(),
            value: initial,
            operation_fn: Box::new(operation_fn),
            percent: false,
        }
    }

    pub fn set_percent(&mut self, flag: bool) {
        self.percent = flag;
    }

    fn magnitude_value(mag: Magnitude) -> f32 {
        match mag {
            Magnitude::Bigger => 1.0,
            Magnitude::Normal => 0.1,
            Magnitude::Smaller => 0.01,
        }
    }
}

impl Operation for Valuef32 {
    fn draw(&self, bounds: Bounds) -> Draw {
        let value = if !self.percent {
            format!("{:+.1}", self.value)
        } else {
            format!("{:+}%", (self.value * 100.0).round() as isize)
        };

        simple(&self.name, value, bounds)
    }

    fn message(&self) -> OperationMessage {
        self.operation_fn.as_ref()(self.value)
    }

    fn action(&mut self, action: Action) {
        match action {
            Action::Increase(mag) => self.value += Self::magnitude_value(mag),
            Action::Decrease(mag) => self.value -= Self::magnitude_value(mag),
            Action::Enter => (),
        }
    }
}

pub struct Toggle {
    name: String,
    flag: bool,
    message: OperationMessage,
}

impl Toggle {
    pub fn new(name: String, flag: bool, message: OperationMessage) -> Self {
        Self {
            name,
            flag,
            message,
        }
    }

    pub fn set(&mut self, flag: bool) {
        self.flag = flag;
    }
}

impl Operation for Toggle {
    fn draw(&self, bounds: Bounds) -> Draw {
        let value = if self.flag { "open" } else { "closed" };
        simple(&self.name, value.into(), bounds)
    }

    fn message(&self) -> OperationMessage {
        self.message.clone()
    }

    fn action(&mut self, action: Action) {
        match action {
            Action::Increase(_) => (),
            Action::Decrease(_) => (),
            Action::Enter => {
                if self.flag {
                    self.flag = false
                } else {
                    self.flag = true
                }
            }
        }
    }
}
