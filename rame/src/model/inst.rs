use crate::error::ParseInstructionError;
use crate::model::Integer;
use std::fmt;
use std::fmt::{Display, Formatter, Write};
use std::num::ParseIntError;
use std::str::FromStr;

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
    #[cfg(feature = "dynamic_jumps")]
    Register(usize),
}

impl<T: Integer> Instruction<T> {
    /// Returns the address targeted by this instruction, if any.
    pub const fn register(&self) -> Option<Register> {
        match *self {
            Instruction::Read | Instruction::Write | Instruction::Stop | Instruction::Nop => None,
            Instruction::Load(v) | Instruction::Add(v) | Instruction::Sub(v) | Instruction::Mul(v) | Instruction::Div(v) | Instruction::Mod(v) => match v {
                Value::Constant(_) => None,
                Value::Register(reg) => Some(reg),
            },
            Instruction::Store(reg) | Instruction::Increment(reg) | Instruction::Decrement(reg) => Some(reg),
            Instruction::Jump(adr) | Instruction::JumpZero(adr) | Instruction::JumpLtz(adr) | Instruction::JumpGtz(adr) => match adr {
                Address::Constant(_) => None,
                #[cfg(feature = "dynamic_jumps")]
                Address::Register(reg) => Some(Register::Direct(reg)),
            },
        }
    }
    
    /// Returns if this instruction is a jump.
    pub const fn is_jump(&self) -> bool {
        match *self {
            Instruction::Jump(_) | Instruction::JumpZero(_) | Instruction::JumpLtz(_) | Instruction::JumpGtz(_) => true,
            Instruction::Read | Instruction::Write | Instruction::Load(_) | Instruction::Add(_) | Instruction::Sub(_) | Instruction::Mul(_) | Instruction::Div(_) | Instruction::Mod(_) | Instruction::Store(_) | Instruction::Increment(_) | Instruction::Decrement(_) | Instruction::Stop | Instruction::Nop => false,
        }
    }
}

impl<T: Integer> FromStr for Instruction<T> {
    type Err = ParseInstructionError<T>;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some((inst, param)) = s.split_once(' ') {
            match inst {
                "LOAD" => Instruction::Load(Value::from_str(param)?),
                "STORE" => Instruction::Store(Register::from_str(param).map_err(ParseInstructionError::InvalidUsize)?),
                "INC" => Instruction::Increment(Register::from_str(param).map_err(ParseInstructionError::InvalidUsize)?),
                "DEC" => Instruction::Decrement(Register::from_str(param).map_err(ParseInstructionError::InvalidUsize)?),
                "ADD" => Instruction::Add(Value::from_str(param)?),
                "SUB" => Instruction::Sub(Value::from_str(param)?),
                "MUL" => Instruction::Mul(Value::from_str(param)?),
                "DIV" => Instruction::Div(Value::from_str(param)?),
                "MOD" => Instruction::Mod(Value::from_str(param)?),
                "JUMP" => Instruction::Jump(Address::from_str(param).map_err(ParseInstructionError::InvalidUsize)?),
                "JUMZ" => Instruction::JumpZero(Address::from_str(param).map_err(ParseInstructionError::InvalidUsize)?),
                "JUML" => Instruction::JumpLtz(Address::from_str(param).map_err(ParseInstructionError::InvalidUsize)?),
                "JUMG" => Instruction::JumpGtz(Address::from_str(param).map_err(ParseInstructionError::InvalidUsize)?),
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
    type Err = ParseInstructionError<T>;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().next().is_some_and(|c| c == '#') {
            T::from_str(&s['#'.len_utf8()..]).map(Value::Constant).map_err(ParseInstructionError::InvalidT)
        } else {
            Register::from_str(s).map(Value::Register).map_err(ParseInstructionError::InvalidUsize)
        }
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

impl Register {
    #[must_use]
    pub const fn adr(&self) -> usize {
        match *self {
            Register::Direct(n) | Register::Indirect(n) => n,
        }
    }
}

impl FromStr for Address {
    type Err = ParseIntError;
    
    #[cfg(feature = "dynamic_jumps")]
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let at = s.chars().next().is_some_and(|c| c == '@');
        if at {
            s = &s['@'.len_utf8()..];
        }
        
        let n = usize::from_str(s)?;
        Ok(if at { Address::Register(n) } else { Address::Constant(n) })
    }
    
    #[cfg(not(feature = "dynamic_jumps"))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        usize::from_str(s).map(Address::Constant)
    }
}

impl Display for Address {
    #[cfg(feature = "dynamic_jumps")]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Address::Register(_) = self {
            f.write_char('@')?;
        }
        
        match self {
            Address::Constant(n) | Address::Register(n) => Display::fmt(n, f),
        }
    }
    
    #[cfg(not(feature = "dynamic_jumps"))]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Address::Constant(n) => Display::fmt(n, f),
        }
    }
}
