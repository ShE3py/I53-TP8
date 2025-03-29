use crate::model::error::ParseCodeError;
use crate::model::{Instruction, Integer, Ir};
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter};
use std::ops::Deref;
use std::path::Path;
use std::str::FromStr;

/// Represents a read-only code segment.
///
/// It may be executed with [`Ram`](crate::runner::Ram), and modified with [`RwCode`](crate::optimizer::WoCode).
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct RoCode<T: Integer>(Vec<Instruction<T>>);

impl<T: Integer> RoCode<T> {
    /// Parses a file.
    /// Blank lines and `; comments` are allowed.
    pub fn parse(f: File) -> Result<RoCode<T>, ParseCodeError<T>> {
        let mut insts = Vec::new();
        
        for (i, l) in BufReader::new(f).lines().enumerate() {
            let l = l?;
            
            // Remove `; comments` and spaces
            let stripped = match l.split_once(';') {
                Some((code, _)) => code,
                None => l.as_str(),
            }.trim_ascii();
            
            if stripped.is_empty() {
                continue;
            }
            
            match Instruction::from_str(stripped) {
                Ok(i) => insts.push(i),
                Err(e) => return Err(ParseCodeError::Inst(i, l, e)),
            }
        }
        
        if insts.is_empty() {
            Err(ParseCodeError::NoInst)
        }
        else {
            Ok(RoCode(insts))
        }
    }
    
    /// Writes `self` into something.
    pub fn write<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(self.to_string().as_bytes())
    }
    
    /// Write `self` into the specified file.
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let f = File::create(path)?;
        self.write(BufWriter::new(f))
    }

    /// Maps a `RoCode<T>` to `RoCode<U>` by applying a function.
    #[must_use]
    #[inline]
    pub fn map<U: Integer, F: Fn(T) -> U>(&self, f: F) -> RoCode<U> {
        RoCode(self.iter().map(|inst| inst.map(&f)).collect())
    }

    /// Maps a `RoCode<T>` to `RoCode<U>`.
    #[must_use]
    #[inline]
    pub fn cast<U: Integer + From<T>>(&self) -> RoCode<U> {
        self.map(<U as From<T>>::from)
    }

    /// Maps a `RoCode<T>` to `RoCode<U>` by applying a function.
    #[inline]
    pub fn try_map<U: Integer, E, F: Fn(T) -> Result<U, E>>(&self, f: F) -> Result<RoCode<U>, E> {
        let mut vec = Vec::with_capacity(self.len());
        for v in self.iter() {
            vec.push(v.try_map(&f)?);
        }
        Ok(RoCode(vec))
    }

    /// Maps a `RoCode<T>` to `RoCode<U>`.
    #[inline]
    pub fn try_cast<U: Integer + TryFrom<T>>(&self) -> Result<RoCode<U>, <U as TryFrom<T>>::Error> {
        self.try_map(U::try_from)
    }
    
    /// Optimize this code using all passes.
    #[inline]
    #[must_use]
    #[cfg(feature = "optimizer")]
    pub fn optimize(&self) -> RoCode<T> {
        crate::optimizer::run_passes(self)
    }

    #[inline]
    #[must_use]
    pub fn get(&self, ir: Ir) -> Option<Instruction<T>> {
        ir.index(self)
    }

    pub fn iter(&self) -> impl Iterator<Item = Instruction<T>> + '_ {
        (**self).iter().copied()
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (Ir, Instruction<T>)> + '_ {
        Ir::enumerate(self.iter())
    }
}

impl<T: Integer> Deref for RoCode<T> {
    type Target = [Instruction<T>];
    
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: Integer> IntoIterator for RoCode<T> {
    type Item = Instruction<T>;
    type IntoIter = <Vec<Instruction<T>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Integer> Display for RoCode<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Some((last, insts)) = self.0.split_last() else {
            return f.write_str("<no code>");
        };
        
        for inst in insts {
            Display::fmt(inst, f)?;
            writeln!(f)?;
        }
        
        Display::fmt(last, f)
    }
}

impl<T: Integer> Default for RoCode<T> {
    /// Returns a program with only a [`STOP` instruction.](`Instruction::Stop`)
    fn default() -> Self {
        RoCode(vec![Instruction::Stop])
    }
}

impl<T: Integer> TryFrom<Vec<Instruction<T>>> for RoCode<T> {
    type Error = ParseCodeError<T>;

    /// Transforms a vector of [`Instruction`]s into a [`RoCode`].
    fn try_from(value: Vec<Instruction<T>>) -> Result<RoCode<T>, ParseCodeError<T>> {
        if value.is_empty() {
            Err(ParseCodeError::NoInst)
        }
        else {
            Ok(RoCode(value))
        }
    }
}

impl<T: Integer> From<&[Instruction<T>]> for RoCode<T> {
    /// Transforms an array of [`Instruction`]s into a [`RoCode`].
    fn from(value: &[Instruction<T>]) -> Self {
        RoCode(value.into())
    }
}

impl<T: Integer, const N: usize> From<[Instruction<T>; N]> for RoCode<T> {
    /// Transforms an array of [`Instruction`]s into a [`RoCode`].
    fn from(value: [Instruction<T>; N]) -> Self {
        RoCode(value.into())
    }
}
