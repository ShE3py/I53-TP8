use crate::error::{print_err, ParseInstructionError};
use std::fmt::{self, Debug, Display, Formatter, Write};
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;

pub mod error;

pub trait Integer: Copy + Clone + Eq + PartialEq + Hash + Default + Debug + Display + FromStr<Err = ParseIntError> {}

impl<T: Copy + Clone + Eq + PartialEq + Hash + Default + Debug + Display + FromStr<Err = ParseIntError>> Integer for T {}

#[derive(Clone, Debug)]
pub struct RoCode<T: Integer>(Vec<Instruction<T>>);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Instruction<T: Integer> {
    Read,
    Write,
    Load(Value<T>),
    Store(Register),
    Increment(Register),
    Decrement(Register),
    Add(Value<T>),
    Sub(Value<T>),
    Mul(Value<T>),
    Div(Value<T>),
    Mod(Value<T>),
    Jump(Address),
    JumpZero(Address),
    JumpLtz(Address),
    JumpGtz(Address),
    Stop,
    #[default] Nop,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Value<T: Integer> {
    Constant(T),
    Register(Register),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Register {
    Direct(usize),
    Indirect(usize),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Address {
    Constant(usize),
    Register(usize),
}

impl<T: Integer> RoCode<T> {
    pub fn parse<P: AsRef<Path>>(path: P) -> RoCode<T> {
        let path = path.as_ref();
        let f = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(concat!(env!("CARGO_PKG_NAME"), ": {}: {}"), path.display(), &e);
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

impl<T: Integer> FromStr for Instruction<T> {
    type Err = ParseInstructionError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some((inst, param)) = s.split_once(' ') {
            match inst {
                "LOAD" => Instruction::Load(Value::from_str(param)?),
                "STORE" => Instruction::Store(Register::from_str(param)?),
                "INC" => Instruction::Increment(Register::from_str(param)?),
                "DEC" => Instruction::Decrement(Register::from_str(param)?),
                "ADD" => Instruction::Add(Value::from_str(param)?),
                "SUB" => Instruction::Sub(Value::from_str(param)?),
                "MUL" => Instruction::Mul(Value::from_str(param)?),
                "DIV" => Instruction::Div(Value::from_str(param)?),
                "MOD" => Instruction::Mod(Value::from_str(param)?),
                "JUMP" => Instruction::Jump(Address::from_str(param)?),
                "JUMZ" => Instruction::JumpZero(Address::from_str(param)?),
                "JUML" => Instruction::JumpLtz(Address::from_str(param)?),
                "JUMG" => Instruction::JumpGtz(Address::from_str(param)?),
                _ => return Err(ParseInstructionError::UnknownInstruction),
            }
        }
        else {
            match s {
                "READ" => Instruction::Read,
                "WRITE" => Instruction::Write,
                "STOP" => Instruction::Stop,
                "NOP" => Instruction::Nop,
                _ => return Err(ParseInstructionError::UnknownInstruction),
            }
        })
    }
}

impl<T: Integer> Display for Instruction<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Read => {
                f.write_str("READ")
            },
            Instruction::Write => {
                f.write_str("WRITE")
            },
            Instruction::Load(v) => {
                f.write_str("LOAD ")?;
                Display::fmt(v, f)
            },
            Instruction::Store(r) => {
                f.write_str("STORE ")?;
                Display::fmt(r, f)
            },
            Instruction::Increment(r) => {
                f.write_str("INC ")?;
                Display::fmt(r, f)
            },
            Instruction::Decrement(r) => {
                f.write_str("DEC ")?;
                Display::fmt(r, f)
            },
            Instruction::Add(v) => {
                f.write_str("ADD ")?;
                Display::fmt(v, f)
            },
            Instruction::Sub(v) => {
                f.write_str("SUB ")?;
                Display::fmt(v, f)
            },
            Instruction::Mul(v) => {
                f.write_str("MUL ")?;
                Display::fmt(v, f)
            },
            Instruction::Div(v) => {
                f.write_str("DIV ")?;
                Display::fmt(v, f)
            },
            Instruction::Mod(v) => {
                f.write_str("MOD ")?;
                Display::fmt(v, f)
            },
            Instruction::Jump(a) => {
                f.write_str("JUMP ")?;
                Display::fmt(a, f)
            },
            Instruction::JumpZero(a) => {
                f.write_str("JUMZ ")?;
                Display::fmt(a, f)
            },
            Instruction::JumpLtz(a) => {
                f.write_str("JUML ")?;
                Display::fmt(a, f)
            },
            Instruction::JumpGtz(a) => {
                f.write_str("JUMG ")?;
                Display::fmt(a, f)
            },
            Instruction::Stop => {
                f.write_str("STOP")
            },
            Instruction::Nop => {
                f.write_str("NOP")
            },
        }
    }
}

impl<T: Integer> FromStr for Value<T> {
    type Err = ParseIntError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.chars().next().is_some_and(|c| c == '#') {
            Value::Constant(T::from_str(&s['#'.len_utf8()..])?)
        } else {
            Value::Register(Register::from_str(s)?)
        })
    }
}

impl<T: Integer> Display for Value<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Constant(n) => {
                f.write_char('#')?;
                Display::fmt(n, f)
            },
            Value::Register(reg) => Display::fmt(reg, f),
        }
    }
}

impl FromStr for Register {
    type Err = ParseIntError;
    
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let at = s.chars().next().is_some_and(|c| c == '@');
        if at {
            s = &s['@'.len_utf8()..];
        }
        
        let n = usize::from_str(s)?;
        Ok(if at { Register::Indirect(n) } else { Register::Direct(n) })
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Register::Indirect(_) = self {
            f.write_char('@')?;
        }
        
        match self {
            Register::Direct(n) | Register::Indirect(n) => Display::fmt(n, f),
        }
    }
}

impl FromStr for Address {
    type Err = ParseIntError;
    
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let at = s.chars().next().is_some_and(|c| c == '@');
        if at {
            s = &s['@'.len_utf8()..];
        }
        
        let n = usize::from_str(s)?;
        Ok(if at { Address::Register(n) } else { Address::Constant(n) })
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Address::Register(_) = self {
            f.write_char('@')?;
        }
        
        match self {
            Address::Constant(n) | Address::Register(n) => Display::fmt(n, f),
        }
    }
}
