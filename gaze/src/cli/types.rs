use std::error::Error;
use std::fmt;
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum OneOrThree<T> {
    One(T),
    Three(T, T, T),
}

impl<T: FromStr> OneOrThree<T> {
    fn generic_from_str(s: &str) -> Result<OneOrThree<&str>, ParseOneOrThreeError> {
        if s.contains(',') {
            // Three values should be present

            let values: Vec<&str> = s.split(',').map(|s| s.trim()).collect();

            if values.len() < 3 {
                return Err(p1o3e_too_little(values.len()));
            } else if values.len() > 3 {
                return Err(p1o3e_too_many(values.len()));
            }

            Ok(OneOrThree::Three(values[0], values[1], values[2]))
        } else {
            // One value should be present

            Ok(OneOrThree::One(s))
        }
    }
}

impl<T: Copy> OneOrThree<T> {
    pub fn as_triple_tuple(&self) -> (T, T, T) {
        match self {
            OneOrThree::One(v) => (*v, *v, *v),
            OneOrThree::Three(a, b, c) => (*a, *b, *c),
        }
    }
}

//TODO: Use  a macro rules to generate these?
impl FromStr for OneOrThree<f32> {
    type Err = ParseOneOrThreeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let strs = Self::generic_from_str(s)?;

        match strs {
            OneOrThree::One(v) => Ok(OneOrThree::One(v.parse()?)),
            OneOrThree::Three(v1, v2, v3) => {
                Ok(OneOrThree::Three(v1.parse()?, v2.parse()?, v3.parse()?))
            }
        }
    }
}

impl FromStr for OneOrThree<u16> {
    type Err = ParseOneOrThreeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let strs = Self::generic_from_str(s)?;

        match strs {
            OneOrThree::One(v) => Ok(OneOrThree::One(v.parse()?)),
            OneOrThree::Three(v1, v2, v3) => {
                Ok(OneOrThree::Three(v1.parse()?, v2.parse()?, v3.parse()?))
            }
        }
    }
}

fn p1o3e_too_little(num: usize) -> ParseOneOrThreeError {
    ParseOneOrThreeError {
        kind: OneOrThreeErrorKind::TooLittleValues(num),
    }
}

fn p1o3e_too_many(num: usize) -> ParseOneOrThreeError {
    ParseOneOrThreeError {
        kind: OneOrThreeErrorKind::TooManyValues(num),
    }
}

fn p1o3e_value_parse(err: &dyn Error) -> ParseOneOrThreeError {
    ParseOneOrThreeError {
        kind: OneOrThreeErrorKind::ValueParseError(
            // FIXME: This produces the debug output?
            // "ParseFloatError { kind: Invalid }" instead of "invalid float literal"
            // from https://doc.rust-lang.org/src/core/num/dec2flt/mod.rs.html#202
            format!("{}", err),
        ),
    }
}

#[derive(Debug)]
pub struct ParseOneOrThreeError {
    kind: OneOrThreeErrorKind,
}

impl Error for ParseOneOrThreeError {}

impl fmt::Display for ParseOneOrThreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            OneOrThreeErrorKind::TooLittleValues(num) => {
                write!(f, "Expected three values, but only saw {}", num)
            }
            OneOrThreeErrorKind::TooManyValues(num) => {
                write!(f, "Only expected three values, but saw {}", num)
            }
            OneOrThreeErrorKind::ValueParseError(err_str) => {
                write!(f, "Failed to parse a value: {}", err_str)
            }
        }
    }
}

impl From<ParseFloatError> for ParseOneOrThreeError {
    fn from(frm: ParseFloatError) -> Self {
        p1o3e_value_parse(&frm)
    }
}

impl From<ParseIntError> for ParseOneOrThreeError {
    fn from(frm: ParseIntError) -> Self {
        p1o3e_value_parse(&frm)
    }
}

#[derive(Debug)]
enum OneOrThreeErrorKind {
    TooLittleValues(usize),
    TooManyValues(usize),
    ValueParseError(String),
}
