use crate::model::Integer;
use any::type_name;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;
use std::{any, fmt};

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

#[derive(Debug)]
pub enum RunError<T: Integer> {
    ReadEof,
    ReadUninit { adr: usize },
    InvalidAddress { adr: T, err: <T as TryInto<usize>>::Error },
    IntegerOverfow,
    InexistentJump,
    InvalidJump { err: <T as TryInto<usize>>::Error },
    Eof,
}

impl<T: Integer> Copy for RunError<T> where <T as TryInto<usize>>::Error: Copy {}

#[expect(clippy::expl_impl_clone_on_copy)]
impl<T: Integer> Clone for RunError<T> where <T as TryInto<usize>>::Error: Clone {
    fn clone(&self) -> Self {
        match self {
            RunError::ReadEof => RunError::ReadEof,
            RunError::ReadUninit { adr } => RunError::ReadUninit { adr: *adr },
            RunError::InvalidAddress { adr, err } => RunError::InvalidAddress { adr: *adr, err: err.clone() },
            RunError::IntegerOverfow => RunError::IntegerOverfow,
            RunError::InexistentJump => RunError::InexistentJump,
            RunError::InvalidJump { err } => RunError::InvalidJump { err: err.clone() },
            RunError::Eof => RunError::Eof,
        }
    }
}

impl<T: Integer> Eq for RunError<T> where <T as TryInto<usize>>::Error: Eq {}

impl<T: Integer> PartialEq for RunError<T> where <T as TryInto<usize>>::Error: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        match self {
            RunError::ReadEof => matches!(other, RunError::ReadEof),
            RunError::ReadUninit { adr } => matches!(other, RunError::ReadUninit { adr: adr1 } if adr == adr1),
            RunError::InvalidAddress { adr, err } => matches!(other, RunError::InvalidAddress { adr: adr1, err: err1 } if adr == adr1 && err == err1),
            RunError::IntegerOverfow => matches!(other, RunError::IntegerOverfow),
            RunError::InexistentJump => matches!(other, RunError::InexistentJump),
            RunError::InvalidJump { err } => matches!(other, RunError::InvalidJump { err: err1 } if err == err1),
            RunError::Eof => matches!(other, RunError::Eof),
        }
    }
}

impl<T: Integer> Display for RunError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RunError::ReadEof => f.write_str("nothing left to read"),
            RunError::ReadUninit { adr } => write!(f, "reading uninitialized memory R{adr}"),
            RunError::InvalidAddress { adr, err } => write!(f, "invalid address R{adr}: {err}"),
            RunError::IntegerOverfow => f.write_str("integer overflow"),
            RunError::InexistentJump => f.write_str("jumping to an inexistent location"),
            RunError::InvalidJump { err } => write!(f, "jumping to an invalid location: {err}"),
            RunError::Eof => f.write_str("unexpected end of file"),
        }
    }
}

impl<T: Integer> Error for RunError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RunError::InvalidAddress { adr: _, err } | RunError::InvalidJump { err } => Some(err),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ParseInstructionError<T: Integer> {
    UnknownInstruction,
    InvalidUsize(<usize as FromStr>::Err),
    InvalidT(<T as FromStr>::Err),
}

impl<T: Integer> Clone for ParseInstructionError<T> where <T as FromStr>::Err: Clone {
    fn clone(&self) -> Self {
        match self {
            ParseInstructionError::UnknownInstruction => ParseInstructionError::UnknownInstruction,
            ParseInstructionError::InvalidUsize(err) => ParseInstructionError::InvalidUsize(err.clone()),
            ParseInstructionError::InvalidT(err) => ParseInstructionError::InvalidT(err.clone()),
        }
    }
}

impl<T: Integer> Eq for ParseInstructionError<T> where <T as FromStr>::Err: Eq {}

impl<T: Integer> PartialEq for ParseInstructionError<T> where <T as FromStr>::Err: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ParseInstructionError::UnknownInstruction => matches!(other, ParseInstructionError::UnknownInstruction),
            ParseInstructionError::InvalidUsize(err) => matches!(other, ParseInstructionError::InvalidUsize(err1) if err == err1),
            ParseInstructionError::InvalidT(err) => matches!(other, ParseInstructionError::InvalidT(err1) if err == err1),
        }
    }
}

impl<T: Integer> Display for ParseInstructionError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseInstructionError::UnknownInstruction => {
                f.write_str("unknown instruction")
            },
            ParseInstructionError::InvalidUsize(e) => {
                f.write_str(concat!("invalid usize: "))?;
                Display::fmt(e, f)
            },
            ParseInstructionError::InvalidT(e) => {
                f.write_fmt(format_args!("invalid {}: ", type_name::<T>()))?;
                Display::fmt(e, f)
            },
        }
    }
}

impl<T: Integer> Error for ParseInstructionError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseInstructionError::UnknownInstruction => None,
            ParseInstructionError::InvalidUsize(e) => Some(e),
            ParseInstructionError::InvalidT(e) => Some(e),
        }
    }
}
