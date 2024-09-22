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
/// It may be executed with [`Ram`](crate::runner::Ram), and modified with [`SeqRewriter`](crate::optimizer::SeqRewriter).
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
        
        Ok(RoCode(insts))
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
    
    #[inline]
    #[must_use]
    pub fn get(&self, ir: Ir) -> Option<Instruction<T>> {
        ir.index(self)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (Ir, Instruction<T>)> + '_ {
        Ir::enumerate((**self).iter().copied())
    }
}

impl<T: Integer> Deref for RoCode<T> {
    type Target = [Instruction<T>];
    
    fn deref(&self) -> &Self::Target {
        self.0.deref()
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
