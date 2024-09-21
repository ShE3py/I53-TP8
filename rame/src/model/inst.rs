use crate::model::{Integer, Ir, ParseInstructionError};
use std::fmt;
use std::fmt::{Display, Formatter, Write};
use std::num::ParseIntError;
use std::str::FromStr;
use sealed::sealed;

/// Represents an instruction.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Instruction<T: Integer> {
    Read,
    Write,
    Load(Value<T>),
    Store(Register<WoLoc>),
    Increment(Register<RwLoc>),
    Decrement(Register<RwLoc>),
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

/// Either the value of a register, or a constant.
/// Read-only memory access.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Value<T: Integer> {
    Constant(T),
    Register(Register<RoLoc>),
}

/// Read-only memory location.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct RoLoc(usize);

/// Write-only memory location.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct WoLoc(usize);

/// Read-write memory location.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct RwLoc(usize);

/// A memory location.
#[sealed]
pub trait Loc: From<usize> + Copy + Display {
    /// The raw address.
    fn raw(self) -> usize;
}

/// The value of a register.
/// Read-write memory access.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Register<L: Loc> {
    /// The value is read/wrote directly into the register.
    Direct(L),
    /// The value is read/wrote from the register this register points to.
    Indirect(RoLoc),
}

/// Where the instructions can jump to.
#[cfg(not(feature = "dynamic_jumps"))]
pub type Address = Ir;

#[cfg(feature = "dynamic_jumps")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Address {
    /// A constant address.
    Constant(Ir),
    /// The value of a register.
    Register(RoLoc),
}

impl<T: Integer> Instruction<T> {
    /// Returns the first address read by this instruction, if any.
    ///
    /// Write-only registers (e.g. [`Instruction::Store`]) are *not* returned.
    #[cfg_attr(feature = "dynamic_jumps", doc = "Indirect jumps registers *are* returned.")]
    pub fn register(&self) -> Option<Register<RoLoc>> {
        match *self {
            Instruction::Read | Instruction::Write | Instruction::Stop | Instruction::Nop => None,
            
            Instruction::Load(v) | Instruction::Add(v) | Instruction::Sub(v) | Instruction::Mul(v) | Instruction::Div(v) | Instruction::Mod(v) => match v {
                Value::Constant(_) => None,
                Value::Register(reg) => Some(reg),
            },
            
            Instruction::Store(_) => None /* write-only */,
            
            Instruction::Increment(reg) | Instruction::Decrement(reg) => Some(reg.downgrade()),
            
            #[cfg(not(feature = "dynamic_jumps"))]
            Instruction::Jump(_) | Instruction::JumpZero(_) | Instruction::JumpLtz(_) | Instruction::JumpGtz(_) => None,
            
            #[cfg(feature = "dynamic_jumps")]
            Instruction::Jump(adr) | Instruction::JumpZero(adr) | Instruction::JumpLtz(adr) | Instruction::JumpGtz(adr) => match adr {
                Address::Constant(_) => None,
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
                "STORE" => Instruction::Store(Register::from_str(param).map_err(ParseInstructionError::InvalidRegister)?),
                "INC" => Instruction::Increment(Register::from_str(param).map_err(ParseInstructionError::InvalidRegister)?),
                "DEC" => Instruction::Decrement(Register::from_str(param).map_err(ParseInstructionError::InvalidRegister)?),
                "ADD" => Instruction::Add(Value::from_str(param)?),
                "SUB" => Instruction::Sub(Value::from_str(param)?),
                "MUL" => Instruction::Mul(Value::from_str(param)?),
                "DIV" => Instruction::Div(Value::from_str(param)?),
                "MOD" => Instruction::Mod(Value::from_str(param)?),
                "JUMP" => Instruction::Jump(Address::from_str(param).map_err(ParseInstructionError::InvalidAddress)?),
                "JUMZ" => Instruction::JumpZero(Address::from_str(param).map_err(ParseInstructionError::InvalidAddress)?),
                "JUML" => Instruction::JumpLtz(Address::from_str(param).map_err(ParseInstructionError::InvalidAddress)?),
                "JUMG" => Instruction::JumpGtz(Address::from_str(param).map_err(ParseInstructionError::InvalidAddress)?),
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
            T::from_str(&s['#'.len_utf8()..]).map(Value::Constant).map_err(ParseInstructionError::InvalidValue)
        } else {
            Register::from_str(s).map(Value::Register).map_err(ParseInstructionError::InvalidRegister)
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

#[sealed]
impl Loc for RoLoc {
    fn raw(self) -> usize { self.0 }
}

#[sealed]
impl Loc for WoLoc {
    fn raw(self) -> usize { self.0 }
}

#[sealed]
impl Loc for RwLoc {
    fn raw(self) -> usize { self.0 }
}

impl From<usize> for RoLoc {
    fn from(loc: usize) -> Self { RoLoc(loc) }
}

impl From<usize> for WoLoc {
    fn from(loc: usize) -> Self { WoLoc(loc) }
}

impl From<usize> for RwLoc {
    fn from(loc: usize) -> Self { RwLoc(loc) }
}

impl From<RwLoc> for RoLoc {
    fn from(loc: RwLoc) -> Self { RoLoc(loc.0) }
}

impl From<RwLoc> for WoLoc {
    fn from(loc: RwLoc) -> Self { WoLoc(loc.0) }
}

impl Display for RoLoc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { Display::fmt(&self.0, f) }
}

impl Display for WoLoc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { Display::fmt(&self.0, f) }
}

impl Display for RwLoc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { Display::fmt(&self.0, f) }
}

impl<L: Loc> FromStr for Register<L> {
    type Err = ParseIntError;
    
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let at = s.chars().next().is_some_and(|c| c == '@');
        if at {
            s = &s['@'.len_utf8()..];
        }
        
        let n = usize::from_str(s)?;
        Ok(if at { Register::Indirect(RoLoc::from(n)) } else { Register::Direct(L::from(n)) })
    }
}

impl<L: Loc> Display for Register<L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Register::Indirect(_) = self {
            f.write_char('@')?;
        }
        
        match self {
            Register::Direct(loc) => Display::fmt(loc, f),
            Register::Indirect(loc) => Display::fmt(loc, f),
        }
    }
}

impl Register<RwLoc> {
    /// Downgrades this register to read-only.
    #[must_use]
    pub fn downgrade(self) -> Register<RoLoc> {
        match self {
            Register::Direct(loc) => Register::Direct(RoLoc::from(loc)),
            Register::Indirect(loc) => Register::Indirect(loc),
        }
    }
}

#[cfg(feature = "dynamic_jumps")]
impl FromStr for Address {
    type Err = ParseIntError;
    
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let at = s.chars().next().is_some_and(|c| c == '@');
        if at {
            s = &s['@'.len_utf8()..];
        }
        
        Ok(if at {
            Address::Register(RoLoc::from(usize::from_str(s)?))
        } else {
            Address::Constant(Ir::from_str(s)?)
        })
    }
}

#[cfg(feature = "dynamic_jumps")]
impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Address::Register(_) = self {
            f.write_char('@')?;
        }
        
        match self {
            Address::Constant(n) => Display::fmt(n, f),
            Address::Register(n) => Display::fmt(n, f),
        }
    }
}

#[cfg(feature = "dynamic_jumps")]
impl From<Ir> for Address {
    fn from(ir: Ir) -> Self {
        Address::Constant(ir)
    }
}
