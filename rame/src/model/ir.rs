use crate::model::{Instruction, Integer};
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::iter;
use std::num::ParseIntError;
use std::ops::{Add, AddAssign};
use std::str::FromStr;

/// Represents the value of a program counter, e.g. the index of an instruction.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(transparent)]
#[must_use]
pub struct Ir(usize);

pub(crate) type Enumerate<T, I> = iter::Map<iter::Enumerate<I>, fn((usize, Instruction<T>)) -> (Ir, Instruction<T>)>;

impl Ir {
    /// Creates a new `Ir` from an `usize`.
    #[inline]
    pub const fn new(ir: usize) -> Ir {
        Ir(ir)
    }
    
    #[inline]
    pub(crate) const fn inner(self) -> usize {
        self.0
    }
    
    /// Checked addition with a signed integer.
    /// Computes `self + rhs`, returning `None` if overflow occured.
    #[inline]
    pub fn checked_add_signed(self, rhs: isize) -> Option<Ir> {
        self.0.checked_add_signed(rhs).map(Ir)
    }
    
    #[inline]
    pub(super) fn index<T: Integer>(self, code: &[Instruction<T>]) -> Option<Instruction<T>> {
        code.get(self.0).copied()
    }
    
    #[inline]
    pub(super) fn enumerate<T: Integer, I: Iterator<Item = Instruction<T>>>(iter: I) -> Enumerate<T, I> {
        iter.enumerate().map(|(i, inst)| (Ir(i), inst))
    }
}

impl FromStr for Ir {
    type Err = ParseIntError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ir(usize::from_str(s)?))
    }
}

impl Display for Ir {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl PartialEq<usize> for Ir {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<usize> for Ir {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        Some(self.0.cmp(other))
    }
}

impl Add<usize> for Ir {
    type Output = Ir;
    
    fn add(self, rhs: usize) -> Self::Output {
        Ir(self.0 + rhs)
    }
}

impl AddAssign<usize> for Ir {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}
