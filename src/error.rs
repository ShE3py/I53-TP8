#![allow(clippy::module_name_repetitions)]

use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::path::Path;

pub(crate) fn print_err(path: &Path, line: &str, line_nb: usize, msg: impl Display) {
    eprintln!(concat!(env!("CARGO_PKG_NAME"), ": {}:{}: {:?}: {}"), path.display(), line_nb + 1, line, msg);
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
