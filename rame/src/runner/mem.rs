use crate::model::{self, Address, Instruction, Integer, Ir, Register, RoLoc, WoLoc, Value, RwLoc};
use crate::runner::{Ram, RunError};
use std::cell::Cell;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub(super) enum Loc<T: Integer> {
    #[default] Uninit,
    Init(T)
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(super) struct LocEntry<'ram, T: Integer, L: model::Loc> {
    pub adr: L,
    pub inner: &'ram Cell<Loc<T>>,
}

impl<'ram, T: Integer> From<LocEntry<'ram, T, RwLoc>> for LocEntry<'ram, T, RoLoc> {
    fn from(loc: LocEntry<'ram, T, RwLoc>) -> Self {
        LocEntry {
            adr: RoLoc::from(loc.adr),
            inner: loc.inner,
        }
    }
}

impl<'ram, T: Integer> From<LocEntry<'ram, T, RwLoc>> for LocEntry<'ram, T, WoLoc> {
    fn from(loc: LocEntry<'ram, T, RwLoc>) -> Self {
        LocEntry {
            adr: WoLoc::from(loc.adr),
            inner: loc.inner,
        }
    }
}

impl<T: Integer> LocEntry<'_, T, RoLoc> {
    pub(super) fn get(&self) -> Result<T, RunError<T>> {
        match self.inner.get() {
            Loc::Uninit => Err(RunError::ReadUninit { adr: self.adr }),
            Loc::Init(v) => Ok(v),
        }
    }
}

impl<T: Integer> LocEntry<'_, T, WoLoc> {
    pub(super) fn set(&self, v: T) {
        self.inner.set(Loc::Init(v));
    }
}

impl<T: Integer> LocEntry<'_, T, RwLoc> {
    pub(super) fn read(&self) -> Result<T, RunError<T>> {
        LocEntry::<'_, T, RoLoc>::from(self.clone()).get()
    }
    
    pub(super) fn write(&self, v: T) {
        LocEntry::<'_, T, WoLoc>::from(self.clone()).set(v);
    }
}

impl<T: Integer> Value<T> {
    /// Fetches the value.
    pub fn get<I: Iterator<Item = T>>(&self, ram: &Ram<T, I>) -> Result<T, RunError<T>> {
        match self {
            Value::Constant(n) => Ok(*n),
            Value::Register(reg) => reg.loc(ram)?.get(),
        }
    }
}

impl<L: model::Loc> Register<L> {
    pub(super) fn loc<'ram, T: Integer, I: Iterator<Item = T>>(&self, ram: &'ram Ram<T, I>) -> Result<LocEntry<'ram, T, L>, RunError<T>> {
        match *self {
            Register::Direct(n) => Ok(ram.loc(n)),
            Register::Indirect(n) => {
                let adr = ram.loc(n).get()?;
                
                match adr.try_into() {
                    Ok(adr) => Ok(ram.loc(L::from(adr))),
                    Err(err) => Err(RunError::InvalidAddress { adr, err }),
                }
            }
        }
    }
}

impl Register<RoLoc> {
    pub fn get<T: Integer, I: Iterator<Item = T>>(&self, ram: &'_ Ram<T, I>) -> Result<T, RunError<T>> {
        self.loc(ram)?.get()
    }
}

impl Register<WoLoc> {
    pub fn set<T: Integer, I: Iterator<Item = T>>(&self, v: T, ram: &'_ Ram<T, I>) -> Result<(), RunError<T>> {
        self.loc(ram)?.set(v);
        Ok(())
    }
}

impl Address {
    #[cfg_attr(not(feature = "indirect_jumps"), expect(clippy::trivially_copy_pass_by_ref))]
    pub(super) fn get<T: Integer, I: Iterator<Item = T>>(&self, ram: &Ram<T, I>) -> Result<(Ir, Instruction<T>), RunError<T>> {
        #[cfg(not(feature = "indirect_jumps"))]
        let ir = *self;

        #[cfg(feature = "indirect_jumps")]
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

impl<T: Integer, L: model::Loc> Display for LocEntry<'_, T, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "R{} = {}", self.adr, self.inner.get())
    }
}
