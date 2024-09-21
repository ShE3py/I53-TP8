use crate::model::{Integer, Ir};
use std::any::type_name;
use std::{fmt, io};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// The error type returned by [`RoCode::parse`](crate::model::RoCode::parse).
#[derive(Debug)]
pub enum ParseCodeError<T: Integer> {
    Io(io::Error),
    Inst(usize, String, ParseInstructionError<T>),
}

impl<T: Integer> Display for ParseCodeError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseCodeError::Io(e) => Display::fmt(e, f),
            ParseCodeError::Inst(i, l, e) => write!(f, "{}: {l:?}: {e}", i + 1),
        }
    }
}

impl<T: Integer> Error for ParseCodeError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseCodeError::Io(e) => Some(e),
            ParseCodeError::Inst(_, _, e) => Some(e),
        }
    }
}

impl<T: Integer> From<io::Error> for ParseCodeError<T> {
    fn from(e: io::Error) -> Self {
        ParseCodeError::Io(e)
    }
}

/// The error type returned by [`Instruction::from_str`](crate::model::Instruction::from_str).
#[derive(Debug)]
pub enum ParseInstructionError<T: Integer> {
    /// The opcode was not recognized.
    UnknownInstruction,
    
    /// Invalid `<value>`.
    InvalidValue(<T as FromStr>::Err),
    
    /// Invalid `<register>`.
    InvalidRegister(<usize as FromStr>::Err),
    
    /// Invalid `<address>`.
    InvalidAddress(<Ir as FromStr>::Err),
}

impl<T: Integer> Clone for ParseInstructionError<T> where <T as FromStr>::Err: Clone {
    fn clone(&self) -> Self {
        match self {
            ParseInstructionError::UnknownInstruction => ParseInstructionError::UnknownInstruction,
            ParseInstructionError::InvalidValue(err) => ParseInstructionError::InvalidValue(err.clone()),
            ParseInstructionError::InvalidRegister(err) => ParseInstructionError::InvalidRegister(err.clone()),
            ParseInstructionError::InvalidAddress(err) => ParseInstructionError::InvalidAddress(err.clone()),
        }
    }
}

impl<T: Integer> Eq for ParseInstructionError<T> where <T as FromStr>::Err: Eq {}

impl<T: Integer> PartialEq for ParseInstructionError<T> where <T as FromStr>::Err: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ParseInstructionError::UnknownInstruction => matches!(other, ParseInstructionError::UnknownInstruction),
            ParseInstructionError::InvalidValue(err) => matches!(other, ParseInstructionError::InvalidValue(err1) if err == err1),
            ParseInstructionError::InvalidRegister(err) => matches!(other, ParseInstructionError::InvalidRegister(err1) if err == err1),
            ParseInstructionError::InvalidAddress(err) => matches!(other, ParseInstructionError::InvalidAddress(err1) if err == err1),
        }
    }
}

impl<T: Integer> Display for ParseInstructionError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseInstructionError::UnknownInstruction => {
                f.write_str("unknown instruction")
            },
            ParseInstructionError::InvalidValue(e) => {
                f.write_fmt(format_args!("invalid `<value>` ({}): ", type_name::<T>()))?;
                Display::fmt(e, f)
            },
            ParseInstructionError::InvalidRegister(e) => {
                f.write_str(concat!("invalid `<register>`: "))?;
                Display::fmt(e, f)
            },
            ParseInstructionError::InvalidAddress(e) => {
                f.write_str(concat!("invalid `<address>`: "))?;
                Display::fmt(e, f)
            },
        }
    }
}

impl<T: Integer> Error for ParseInstructionError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseInstructionError::UnknownInstruction => None,
            ParseInstructionError::InvalidValue(e) => Some(e),
            ParseInstructionError::InvalidRegister(e) | ParseInstructionError::InvalidAddress(e) => Some(e),
        }
    }
}

