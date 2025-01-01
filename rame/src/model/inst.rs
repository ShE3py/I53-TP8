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
#[cfg(not(feature = "indirect_jumps"))]
pub type Address = Ir;

#[cfg(feature = "indirect_jumps")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Address {
    /// A constant address.
    Constant(Ir),
    /// The value of a register.
    Register(RoLoc),
}

impl<T: Integer> Instruction<T> {
    /// Returns the value read by this instruction, if any.
    #[must_use] #[inline]
    pub const fn value(self) -> Option<Value<T>> {
        match self {
            Instruction::Load(v) | Instruction::Add(v) | Instruction::Sub(v) | Instruction::Mul(v) | Instruction::Div(v) | Instruction::Mod(v) => Some(v),
            _ => None,
        }
    }
    
    /// Returns the first register read by this instruction, if any.
    #[cfg_attr(feature = "indirect_jumps", doc = "Indirect jumps registers *are* returned.")]
    #[must_use] #[inline]
    pub const fn register(self) -> Option<Register<RoLoc>> {
        match self {
            Instruction::Load(v) | Instruction::Add(v) | Instruction::Sub(v) | Instruction::Mul(v) | Instruction::Div(v) | Instruction::Mod(v) => match v {
                Value::Constant(_) => None,
                Value::Register(reg) => Some(reg),
            },
            
            Instruction::Increment(reg) | Instruction::Decrement(reg) => Some(reg.downgrade()),

            #[cfg(feature = "indirect_jumps")]
            Instruction::Jump(adr) | Instruction::JumpZero(adr) | Instruction::JumpLtz(adr) | Instruction::JumpGtz(adr) => match adr {
                Address::Constant(_) => None,
                Address::Register(reg) => Some(Register::Direct(reg)),
            },
            
            _ => None,
        }
    }
    
    /// Returns the address this instruction jumps to, if any.
    #[must_use] #[inline]
    pub const fn jump(self) -> Option<Address> {
        match self {
            Instruction::Jump(adr) | Instruction::JumpZero(adr) | Instruction::JumpLtz(adr) | Instruction::JumpGtz(adr) => Some(adr),
            _ => None,
        }
    }
}

impl<T: Integer> FromStr for Instruction<T> {
    type Err = ParseInstructionError<T>;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[inline]
        fn parse_register<L: Loc, T: Integer>(s: &str) -> Result<Register<L>, ParseInstructionError<T>> {
            Register::from_str(s).map_err(ParseInstructionError::InvalidRegister)
        }

        #[inline]
        fn parse_addr<T: Integer>(s: &str) -> Result<Address, ParseInstructionError<T>> {
            // Manual check as `Ir::from_str` doesn't check
            if !cfg!(feature = "indirect_jumps") && s.starts_with('@') {
                return Err(ParseInstructionError::DisabledIndirect);
            }

            Address::from_str(s).map_err(ParseInstructionError::InvalidAddress)
        }

        Ok(if let Some((inst, param)) = s.split_once(' ') {
            match inst {
                "LOAD" => Instruction::Load(Value::from_str(param)?),
                "STORE" => Instruction::Store(parse_register(param)?),
                "INC" => Instruction::Increment(parse_register(param)?),
                "DEC" => Instruction::Decrement(parse_register(param)?),
                "ADD" => Instruction::Add(Value::from_str(param)?),
                "SUB" => Instruction::Sub(Value::from_str(param)?),
                "MUL" => Instruction::Mul(Value::from_str(param)?),
                "DIV" => Instruction::Div(Value::from_str(param)?),
                "MOD" => Instruction::Mod(Value::from_str(param)?),
                "JUMP" => Instruction::Jump(parse_addr(param)?),
                "JUMZ" => Instruction::JumpZero(parse_addr(param)?),
                "JUML" => Instruction::JumpLtz(parse_addr(param)?),
                "JUMG" => Instruction::JumpGtz(parse_addr(param)?),
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

#[test]
#[cfg_attr(not(feature = "indirect_jumps"), should_panic = "the `indirect_jumps` feature is opted out")]
fn parse_indirect() {
    Instruction::<i16>::from_str("JUML @4").map_err(|e| format!("{e}")).unwrap();
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

impl RoLoc {
    const fn from_const(loc: RwLoc) -> Self { RoLoc(loc.0) }
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
    pub const fn downgrade(self) -> Register<RoLoc> {
        match self {
            Register::Direct(loc) => Register::Direct(RoLoc::from_const(loc)),
            Register::Indirect(loc) => Register::Indirect(loc),
        }
    }
}

#[cfg(feature = "indirect_jumps")]
impl FromStr for Address {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some(s) = s.strip_prefix('@') {
            Address::Register(RoLoc::from(usize::from_str(s)?))
        } else {
            Address::Constant(Ir::from_str(s)?)
        })
    }
}

#[cfg(feature = "indirect_jumps")]
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

#[cfg(feature = "indirect_jumps")]
impl From<Ir> for Address {
    fn from(ir: Ir) -> Self {
        Address::Constant(ir)
    }
}
