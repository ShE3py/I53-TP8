use crate::model::{Integer, RoLoc};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

/// The error type returned by [`Ram::step`](crate::runner::Ram::step).
#[derive(Debug)]
pub enum RunError<T: Integer> {
    /// A [`Instruction::Read`](crate::model::Instruction::Read) was issued, but nothing was left to read.
    ReadEof,
    
    /// An unitialized memory location was read.
    ReadUninit { adr: RoLoc },
    
    /// A [`Register::Indirect`](crate::model::Register::Indirect) read/write was attempted,
    /// but the intermediate register's value wasn't a valid address.
    InvalidAddress { adr: T, err: <T as TryInto<usize>>::Error },
    
    /// An arithmetic instruction overflowed.
    IntegerOverfow,
    
    /// A jump instruction jumped to an inexistent [`Ir`](crate::model::Ir).
    InexistentJump,
    
    /// An [`Address::Register`] jump was attempted,
    /// but the register's value wasn't a valid [`Ir`](crate::model::Ir).
    #[cfg(feature = "indirect_jumps")]
    InvalidJump { err: <T as TryInto<usize>>::Error },
    
    /// [`Ram::step`](crate::runner::Ram::step) was called, even though there's no
    /// instruction left to execute.
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
            #[cfg(feature = "indirect_jumps")] RunError::InvalidJump { err } => RunError::InvalidJump { err: err.clone() },
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
            #[cfg(feature = "indirect_jumps")]  RunError::InvalidJump { err } => matches!(other, RunError::InvalidJump { err: err1 } if err == err1),
            RunError::Eof => matches!(other, RunError::Eof),
        }
    }
}

impl<T: Integer> Display for RunError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RunError::ReadEof => f.write_str("nothing left to read"),
            RunError::ReadUninit { ref adr } => write!(f, "reading uninitialized memory R{adr}"),
            RunError::InvalidAddress { ref adr, err } => write!(f, "invalid address R{adr}: {err}"),
            RunError::IntegerOverfow => f.write_str("integer overflow"),
            RunError::InexistentJump => f.write_str("jumping to an inexistent location"),
            #[cfg(feature = "indirect_jumps"
            )]  RunError::InvalidJump { err } => write!(f, "jumping to an invalid location: {err}"),
            RunError::Eof => f.write_str("unexpected end of file"),
        }
    }
}

impl<T: Integer> Error for RunError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RunError::InvalidAddress { adr: _, err } => Some(err),
            #[cfg(feature = "indirect_jumps")]  RunError::InvalidJump { err } => Some(err),
            _ => None,
        }
    }
}
