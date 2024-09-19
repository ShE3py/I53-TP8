#![allow(clippy::module_name_repetitions)]

use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::path::Path;
use crate::Integer;

pub(crate) fn format_err(path: &Path, line: &str, line_nb: usize, msg: impl Display) -> String {
    if !line.is_empty() {
        format!("error: {}:{}: {:?}: {}", path.display(), line_nb + 1, line, msg)
    } else {
        format!("error: {}:{}: {}", path.display(), line_nb + 1, msg)
    }
}

pub(crate) fn format_help(path: &Path, line_nb: usize, msg: impl Display) -> String {
    format_err(path, "", line_nb, format!("help: {msg}"))
}

pub(crate) fn print_err(path: &Path, line: &str, line_nb: usize, msg: impl Display) {
    eprintln!("{}", format_err(path, line, line_nb, msg));
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum RunError<T: Integer> {
    ReadEof,
    ReadUninit { adr: usize },
    InvalidAddress { adr: T, err: <T as TryInto<usize>>::Error },
    IntegerOverfow,
    InexistentJump,
    Eof,
}

impl<T: Integer> Display for RunError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RunError::ReadEof => f.write_str("nothing left to read"),
            RunError::ReadUninit { adr } => write!(f, "reading uninitialized memory R{adr}"),
            RunError::InvalidAddress { adr, err } => write!(f, "invalid address R{adr}: {err}"),
            RunError::IntegerOverfow => f.write_str("integer overflow"),
            RunError::InexistentJump => f.write_str("jumping to an inexistent location"),
            RunError::Eof => f.write_str("unexpected end of file"),
        }
    }
}

impl<T: Integer> Error for RunError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RunError::InvalidAddress { adr: _, err } => Some(err),
            _ => None,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ParseInstructionError {
    UnknownInstruction,
    InvalidParameter(ParseIntError),
}

impl Display for ParseInstructionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseInstructionError::UnknownInstruction => {
                f.write_str("unknown instruction")
            },
            ParseInstructionError::InvalidParameter(e) => {
                f.write_str("invalid parameter: ")?;
                Display::fmt(e, f)
            },
        }
    }
}

impl Error for ParseInstructionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseInstructionError::UnknownInstruction => None,
            ParseInstructionError::InvalidParameter(e) => Some(e),
        }
    }
}

impl From<ParseIntError> for ParseInstructionError {
    fn from(e: ParseIntError) -> Self {
        ParseInstructionError::InvalidParameter(e)
    }
}
