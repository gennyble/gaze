use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Attribute {
    Hue,
    Saturation,
    Value,
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Attribute::Hue => write!(f, "hue"),
            Attribute::Saturation => write!(f, "saturation"),
            Attribute::Value => write!(f, "value"),
        }
    }
}

impl From<Attribute> for usize {
    fn from(a: Attribute) -> usize {
        match a {
            Attribute::Hue => 0,
            Attribute::Saturation => 1,
            Attribute::Value => 2,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    Red,
    Green,
    Blue,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Red => write!(f, "red"),
            Color::Green => write!(f, "green"),
            Color::Blue => write!(f, "blue"),
        }
    }
}

impl From<Color> for usize {
    fn from(c: Color) -> usize {
        match c {
            Color::Red => 0,
            Color::Green => 1,
            Color::Blue => 2,
        }
    }
}
