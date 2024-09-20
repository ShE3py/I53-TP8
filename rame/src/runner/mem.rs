use crate::error::RunError;
use crate::model::{Address, Instruction, Integer, Ir, Register, Value};
use crate::runner::Ram;
use std::cell::Cell;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub(super) enum Loc<T: Integer> {
    #[default] Uninit,
    Init(T)
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(super) struct LocEntry<'ram, T: Integer> {
    pub adr: usize,
    pub inner: &'ram Cell<Loc<T>>,
}

impl<T: Integer> LocEntry<'_, T> {
    pub(super) fn set(&self, v: T) {
        self.inner.set(Loc::Init(v));
    }
    
    pub(super) fn get(&self) -> Result<T, RunError<T>> {
        match self.inner.get() {
            Loc::Uninit => Err(RunError::ReadUninit { adr: self.adr }),
            Loc::Init(v) => Ok(v),
        }
    }
}

impl<T: Integer> Value<T> {
    pub fn get<I: Iterator<Item = T>>(&self, ram: &mut Ram<T, I>) -> Result<T, RunError<T>> {
        match self {
            Value::Constant(n) => Ok(*n),
            Value::Register(reg) => reg.get(ram),
        }
    }
}

impl Register {
    pub(super) fn loc<'ram, T: Integer, I: Iterator<Item = T>>(&self, ram: &'ram Ram<T, I>) -> Result<LocEntry<'ram, T>, RunError<T>> {
        match *self {
            Register::Direct(n) => Ok(ram.loc(n)),
            Register::Indirect(n) => {
                let adr = ram.loc(n).get()?;
                
                match adr.try_into() {
                    Ok(adr) => Ok(ram.loc(adr)),
                    Err(err) => Err(RunError::InvalidAddress { adr, err }),
                }
            }
        }
    }
    
    pub fn set<T: Integer, I: Iterator<Item = T>>(&self, v: T, ram: &mut Ram<T, I>) -> Result<(), RunError<T>> {
        self.loc(ram)?.set(v);
        Ok(())
    }
    
    pub fn get<T: Integer, I: Iterator<Item = T>>(&self, ram: &mut Ram<T, I>) -> Result<T, RunError<T>> {
        self.loc(ram)?.get()
    }
}

impl Address {
    #[cfg_attr(not(feature = "dynamic_jumps"), expect(clippy::trivially_copy_pass_by_ref))]
    pub(super) fn get<T: Integer, I: Iterator<Item = T>>(&self, ram: &Ram<T, I>) -> Result<(Ir, Instruction<T>), RunError<T>> {
        #[cfg(not(feature = "dynamic_jumps"))]
        let ir = *self;
        
        #[cfg(feature = "dynamic_jumps")]
        let ir = match *self {
            Address::Constant(adr) => adr,
            Address::Register(adr) => {
                let adr = ram.loc(adr).get()?;
                adr.try_into().map(Ir::new).map_err(|err| RunError::InvalidJump { err })?
            }
        };
        
        
        ram.code.get(ir).map_or(Err(RunError::InexistentJump), |inst| Ok((ir, inst)))
    }
}

impl<T: Integer> Display for Loc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Loc::Uninit => f.write_str("<uninitialized>"),
            Loc::Init(v) => Display::fmt(v, f),
        }
    }
}

impl<T: Integer> Display for LocEntry<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "R{} = {}", self.adr, self.inner.get())
    }
}
