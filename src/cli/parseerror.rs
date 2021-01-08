use super::ParseOneOrThreeError;
use std::num::{ParseFloatError, ParseIntError};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind
}

//TODO: source
impl Error for ParseError{}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ParseErrorKind::Int(err) => {
                write!(f, "Failed to parse int: {}\n\nIntegers are whole numbers. Ex: 1, 5, or -1", err)
            },
            ParseErrorKind::Float(err) => {
                write!(f, "Failed to parse float: {}\n\nFloats are numbers. Ex: 1.0, 1, -1, or -1.0", err)
            },
            ParseErrorKind::OneOrThree(err) => {
                write!(
                        f,
                        "Failed to parse value: {}\n\n\
                        You can use 1 number, or three seperated my a comma.\n\
                        Ex:\n\t\
                            1 or 1,4,2 or \"1, 4, 2\"",
                        err
                    )
            }
        }
    }
}

#[derive(Debug)]
pub enum ParseErrorKind {
    Int(ParseIntError),
    Float(ParseFloatError),
    OneOrThree(ParseOneOrThreeError)
}

//TODO: Macro rules?
impl From<ParseIntError> for ParseError {
    fn from(frm: ParseIntError) -> Self {
        ParseError { kind: ParseErrorKind::Int(frm) }
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(frm: ParseFloatError) -> Self {
        ParseError { kind: ParseErrorKind::Float(frm) }
    }
}

impl From<ParseOneOrThreeError> for ParseError {
    fn from(frm: ParseOneOrThreeError) -> Self {
        ParseError { kind: ParseErrorKind::OneOrThree(frm) }
    }
}