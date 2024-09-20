use std::fmt::{Display, Formatter};
use std::fs::File;
use std::{fmt, io};
use std::io::{BufRead, BufReader, BufWriter};
use std::ops::Deref;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use crate::error::print_err;
use crate::model::{Instruction, Integer, Ir};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct RoCode<T: Integer>(Vec<Instruction<T>>);

impl<T: Integer> RoCode<T> {
    pub fn parse<P: AsRef<Path>>(path: P) -> RoCode<T> {
        let path = path.as_ref();
        let f = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("error: {path:?}: {e}");
                exit(1);
            },
        };
        
        let mut insts = Vec::new();
        let mut errs = 0;
        
        for (i, l) in BufReader::new(f).lines().enumerate() {
            let l = match l {
                Ok(l) => l,
                Err(e) => {
                    print_err(path, "<err>", i, e);
                    errs += 1;
                    continue;
                },
            };
            
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
                Err(e) => {
                    print_err(path, &l, i, e);
                    errs += 1;
                    continue;
                }
            }
        }
        
        if errs != 0 {
            exit(1);
        }
        
        RoCode(insts)
    }
    
    pub fn write<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(self.to_string().as_bytes())
    }
    
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

impl<T: Integer> From<&[Instruction<T>]> for RoCode<T> {
    fn from(value: &[Instruction<T>]) -> Self {
        RoCode(value.into())
    }
}

impl<T: Integer, const N: usize> From<[Instruction<T>; N]> for RoCode<T> {
    fn from(value: [Instruction<T>; N]) -> Self {
        RoCode(value.into())
    }
}

impl<T: Integer> Default for RoCode<T> {
    fn default() -> Self {
        RoCode(vec![Instruction::Stop])
    }
}
