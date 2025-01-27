//! An emulator for RAM programs.

use crate::error::{format_err, format_help};
use crate::model::{self, Address, Instruction, Integer, Ir, ParseCodeError, Register, RoCode, RoLoc, RwLoc, Value};
use crate::runner::mem::{Loc, LocEntry};
use std::cell::{Cell, UnsafeCell};
use std::hint::assert_unchecked;
use std::iter::{self, Fuse};
use std::process::exit;

mod error;
mod mem;

pub use error::RunError;

type Memory<T> = UnsafeCell<Vec<Cell<Loc<T>>>>;

/// A process for a RAM program.
///
/// The type parameter `I` is an iterator consumed by [`Instruction::Read`],
/// allowing input to be read lazily (e.g. interactivly through a terminal).
///
/// # Example
///
/// ```
/// # use rame::inst;
/// # use rame::model::RoCode;
/// # use rame::runner::Ram;
/// let code = RoCode::from([
///     inst!(READ),
///     inst!(ADD #2),
///     inst!(WRITE),
///     inst!(STOP),
/// ]);
/// let ram = Ram::new(code, [1]);
/// assert_eq!(ram.run(), [3]);
/// ```
#[derive(Debug)]
#[must_use]
pub struct Ram<T: Integer, I: Iterator<Item = T>> {
    input: Fuse<I>,
    output: Vec<T>,
    memory: Memory<T>,
    code: RoCode<T>,
    
    /// The next instruction to run.
    inst: Instruction<T>,
    /// Instruction register (the index of `inst`).
    ir: Ir,
}

impl<T: Integer, I: Iterator<Item = T>> Ram<T, I> {
    /// Creates a new `Ram` from its source code and input.
    pub fn new(code: RoCode<T>, input: impl IntoIterator<IntoIter = I>) -> Ram<T, I> {
        let Some(inst) = code.first().copied() else {
            if cfg!(test) {
                panic!("{}", ParseCodeError::<T>::NoInst);
            }
            else {
                eprintln!("error: {}", ParseCodeError::<T>::NoInst);
                exit(1);
            }
        };
        
        Ram {
            input: input.into_iter().fuse(),
            output: Vec::default(),
            memory: Memory::default(),
            code,
            inst,
            ir: Ir::default(),
        }
    }
    
    /// Executes the next instruction.
    pub fn step(&mut self) -> Result<(), RunError<T>> {
        match self.inst {
            Instruction::Read => {
                let Some(v) = self.input.next() else {
                    return Err(RunError::ReadEof);
                };
                
                self.acc().set(v);
            },
            Instruction::Write => {
                let acc = self.acc().get()?;
                self.output.push(acc);
            },
            Instruction::Load(v) => {
                let v = v.get(self)?;
                self.acc().set(v);
            },
            Instruction::Store(reg) => {
                let acc = self.acc().get()?;
                reg.set(acc, self)?;
            }
            Instruction::Increment(reg) => {
                self.unop(reg, T::checked_add)?;
            }
            Instruction::Decrement(reg) => {
                self.unop(reg, T::checked_sub)?;
            }
            Instruction::Add(v) => {
                self.binop(v, T::checked_add)?;
            },
            Instruction::Sub(v) => {
                self.binop(v, T::checked_sub)?;
            },
            Instruction::Mul(v) => {
                self.binop(v, T::checked_mul)?;
            },
            Instruction::Div(v) => {
                self.binop(v, T::checked_div)?;
            },
            Instruction::Mod(v) => {
                self.binop(v, T::checked_rem)?;
            },
            Instruction::Jump(addr) => {
                return self.jump(addr);
            }
            Instruction::JumpZero(addr) => {
                if self.acc().get()?.is_zero() {
                    return self.jump(addr);
                }
            },
            Instruction::JumpLtz(addr) => {
                if self.acc().get()? < T::zero() {
                    return self.jump(addr);
                }
            },
            Instruction::JumpGtz(addr) => {
                if self.acc().get()? > T::zero() {
                    return self.jump(addr);
                }
            },
            Instruction::Stop => {
                return Ok(());
            }
            Instruction::Nop => {}
        }
        
        self.ir += 1;
        match self.code.get(self.ir) {
            Some(inst) => {
                self.inst = inst;
                Ok(())
            },
            None => Err(RunError::Eof),
        }
    }
    
    /// Either `INC` or `DEC`.
    fn unop<F: Fn(&T, &T) -> Option<T>>(&self, reg: Register<RwLoc>, f: F) -> Result<(), RunError<T>> {
        let loc = reg.loc(self)?;
        let v = loc.read()?;
        f(&v, &T::one()).map(|r| loc.write(r)).ok_or(RunError::IntegerOverfow)
    }
    
    /// Arithmetic instructions on ACC.
    fn binop<F: Fn(&T, &T) -> Option<T>>(&self, v: Value<T>, f: F) -> Result<(), RunError<T>> {
        let acc = self.acc().get()?;
        let v = v.get(self)?;
        f(&acc, &v).map(|r| self.acc().set(r)).ok_or(RunError::IntegerOverfow)
    }
    
    /// Jumps at the specified address, updating the current instruction.
    fn jump(&mut self, adr: Address) -> Result<(), RunError<T>> {
        let (ir, inst) = adr.get(self)?;
        
        self.ir = ir;
        self.inst = inst;
        Ok(())
    }
    
    /// Runs the whole program, and returns its output.
    pub fn run(mut self) -> Vec<T> {
        loop {
            let ir = self.ir;
            
            match self.step() {
                Ok(()) if self.inst == Instruction::Stop => break self.output,
                Ok(()) => continue,
                Err(e) => self.emit_err(ir, e),
            }
        }
    }
    
    #[expect(clippy::needless_pass_by_value)]
    fn emit_err(&self, ir: Ir, e: RunError<T>) -> ! {
        let path = "anon".as_ref();
        let inst = self.code.get(ir);
        let snip = inst.map(|inst| inst.to_string()).unwrap_or_default();
        let mut err = format_err(path, &snip, ir.inner(), e.to_string());
        
        if !matches!(e, RunError::Eof | RunError::ReadUninit { .. }) {
            if let Some(inst) = inst {
                // Show ACC value
                if inst.should_print_acc() {
                    err.push('\n');
                    err.push_str(&format_help(path, ir.inner(), format!("ACC = {}", self.acc::<RoLoc>().inner.get())));
                }
                
                // Show register value
                match inst.register() {
                    Some(Register::Direct(adr)) if !matches!(e, RunError::ReadUninit { .. }) => {
                        let loc = self.loc(adr);
                        
                        err.push('\n');
                        err.push_str(&format_help(path, ir.inner(), loc));
                    },
                    Some(Register::Indirect(adr)) => {
                        let val = self.loc(adr).get().unwrap();
                        let loc = match val.try_into() {
                            Ok(adr) => Ok(self.loc(RoLoc::from(adr))),
                            Err(err) => Err(RunError::InvalidAddress { adr: val, err }),
                        };
                        
                        let msg = loc.map_or_else(|err| format!("<{err}>"), |val| val.to_string());
                        
                        err.push('\n');
                        err.push_str(&format_help(path, ir.inner(), msg));
                        
                    },
                    _ => {},
                }
            }
        }
        
        match e {
            RunError::IntegerOverfow => {
                err.push('\n');
                err.push_str(&format_help(path, ir.inner(), format!("using `--bits={}`; only values from {} to {} are accepted.", size_of::<T>() * 8, &T::min_value(), &T::max_value())));
            },
            RunError::Eof => {
                err.push('\n');
                err.push_str(&format_help(path, ir.inner(), "missing `STOP`?"));
            },
            _ => {}
        }
        
        if cfg!(test) { panic!("{err}") } else { eprintln!("{err}") };
        exit(1);
    }
    
    /// Returns `self`'s current output.
    #[inline]
    pub fn output(&self) -> &[T] {
        &self.output
    }
    
    fn loc<L: model::Loc>(&self, adr: L) -> LocEntry<'_, T, L> {
        // SAFETY: this function is the only one that uses `self.memory`,
        //  we don't call code that could call this function a 2nd time,
        //  so there's no references that point to our emulated memory.
        let memory = unsafe { &mut *self.memory.get() };
        
        let raw_adr = adr.raw();
        if raw_adr >= memory.len() {
            #[cold]
            #[inline(never)]
            fn resize_mem<T: Integer>(memory: &mut Vec<Cell<Loc<T>>>, new_len: usize) {
                memory.resize(new_len, Cell::new(Loc::Uninit));
            }
            
            resize_mem(memory, raw_adr + 1);
        };
        
        // SAFETY: if `adr >= memory.len()`, we resize so that `memory.len() == adr + 1`,
        //  so `memory.len() > adr`.
        unsafe { assert_unchecked(raw_adr < memory.len()) };
        
        LocEntry {
            adr,
            inner: &memory[raw_adr],
        }
    }
    
    #[inline]
    fn acc<L: model::Loc>(&self) -> LocEntry<'_, T, L> {
        self.loc(L::from(0))
    }
    
    /// Returns `self`'s source code.
    #[inline]
    pub const fn code(&self) -> &RoCode<T> {
        &self.code
    }
}

impl<T: Integer, I: Iterator<Item = T> + Default> Default for Ram<T, I> {
    /// Returns a process with only a [`STOP` instruction,](`Instruction::Stop`)
    /// and `I::default()` input.
    fn default() -> Self {
        Ram {
            input: I::default().fuse(),
            output: Vec::default(),
            memory: Memory::default(),
            code: RoCode::default(),
            inst: Instruction::Stop,
            ir: Ir::default(),
        }
    }
}

impl<T: Integer> Ram<T, iter::Empty<T>> {
    /// Returns a process with only a [`STOP` instruction,](`Instruction::Stop`)
    /// and no input.
    pub fn empty() -> Self {
        Self::default()
    }
    
    /// Creates a process for the specified source code,
    /// and no input.
    pub fn without_inputs(code: RoCode<T>) -> Self {
        Self::new(code, iter::empty())
    }
}

impl<T: Integer> From<RoCode<T>> for Ram<T, iter::Empty<T>> {
    /// Creates a process for the specified source code,
    /// and no input.
    fn from(code: RoCode<T>) -> Self {
        Self::without_inputs(code)
    }
}

impl<T: Integer> From<&[Instruction<T>]> for Ram<T, iter::Empty<T>> {
    /// Creates a process for the specified source code,
    /// and no input.
    fn from(code: &[Instruction<T>]) -> Self {
        Self::without_inputs(code.into())
    }
}

impl<T: Integer, const N: usize> From<[Instruction<T>; N]> for Ram<T, iter::Empty<T>> {
    /// Creates a process for the specified source code,
    /// and no input.
    fn from(code: [Instruction<T>; N]) -> Self {
        Self::without_inputs(code.into())
    }
}

impl<T: Integer> Instruction<T> {
    pub(crate) const fn should_print_acc(&self) -> bool {
        match self {
            Instruction::Add(_) | Instruction::Sub(_) | Instruction::Mul(_) | Instruction::Div(_) | Instruction::Mod(_)| Instruction::JumpZero(_) | Instruction::JumpLtz(_) | Instruction::JumpGtz(_) => true,
            Instruction::Read | Instruction::Write | Instruction::Load(_) | Instruction::Store(_) | Instruction::Increment(_) | Instruction::Decrement(_) | Instruction::Jump(_)  | Instruction::Stop | Instruction::Nop => false,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::runner::Ram;
    use crate::{inst, ram};
    
    #[test]
    #[should_panic = "empty file"]
    fn run_no_inst() {
        Ram::<i32, _>::run([].into());
    }
    
    #[test]
    #[should_panic = "nothing left to read"]
    fn read_eof() {
        ram!(READ).run();
    }
    
    #[test]
    #[should_panic = "reading uninitialized memory R0"]
    fn write_uninit() {
        ram!(WRITE).run();
    }
    
    #[test]
    fn read_write() {
        assert_eq!(ram!(12; READ WRITE STOP).run(), [12]);
    }
    
    #[test]
    #[should_panic = "reading uninitialized memory R0"]
    fn load_uninit() {
        Ram::<i32, _>::run([
            inst!(LOAD 0)
        ].into());
    }
    
    #[test]
    fn load_write() {
        let ram: Ram<_, _> = [
            inst!(LOAD #-1),
            inst!(WRITE),
            inst!(STOP)
        ].into();
        
        assert_eq!(ram.run(), [-1]);
    }
    
    #[test]
    #[should_panic = "reading uninitialized memory R1"]
    fn load_indirect_uninit() {
        Ram::<i32, _>::run([
            inst!(LOAD #1),
            inst!(LOAD @0)
        ].into());
    }
    
    #[test]
    #[should_panic = "invalid address R-10"]
    fn load_indirect_negative() {
        Ram::<i32, _>::run([
            inst!(LOAD #-10),
            inst!(LOAD @0)
        ].into());
    }
    
    #[test]
    #[should_panic = "invalid address R-3"]
    fn store_indirect_negative() {
        Ram::<i32, _>::run([
            inst!(LOAD #-3),
            inst!(STORE 1),
            inst!(STORE @1)
        ].into());
    }
    
    #[test]
    #[should_panic = "integer overflow"]
    fn dec_overflow() {
        Ram::<u32, _>::run([
            inst!(LOAD #0),
            inst!(DEC 0),
        ].into());
    }
    
    #[test]
    #[should_panic = "integer overflow"]
    fn inc_overflow() {
        Ram::<u8, _>::run([
            inst!(LOAD #255),
            inst!(INC 0),
        ].into());
    }
    
    #[test]
    #[should_panic = "integer overflow"]
    fn add_overflow() {
        Ram::<u8, _>::run([
            inst!(LOAD #200),
            inst!(STORE 1),
            inst!(LOAD #70),
            inst!(ADD 1),
        ].into());
    }
    
    #[test]
    #[should_panic = "integer overflow"]
    fn div_zero() {
        Ram::<u8, _>::run([
            inst!(LOAD #1),
            inst!(DIV #0),
        ].into());
    }

    #[test]
    #[should_panic = "integer overflow"]
    fn rem_zero() {
        Ram::<u8, _>::run([
            inst!(LOAD #1),
            inst!(MOD #0),
        ].into());
    }

    #[test]
    fn rem() {
        // -5 % 2,
        // -5 % -2
        let ram: Ram<i8, _> = [
            inst!(LOAD #2),
            inst!(STORE 1),
            inst!(LOAD #-5),
            inst!(STORE 2),
            inst!(MOD 1),
            inst!(WRITE),
            inst!(LOAD #-2),
            inst!(STORE 1),
            inst!(LOAD 2),
            inst!(MOD 1),
            inst!(WRITE),
            inst!(STOP)
        ].into();
        
        assert_eq!(ram.run(), [-1, -1]);
    }
    
    #[test]
    fn jump() {
        let ram: Ram<_, _> = [
            inst!(LOAD #1),
            inst!(JUMP 3),
            inst!(LOAD #0),
            inst!(WRITE),
            inst!(STOP)
        ].into();
        
        assert_eq!(ram.run(), [1]);
    }
    
    #[test]
    #[should_panic = "jumping to an inexistent location"]
    fn jump_inexistent() {
        Ram::<i32, _>::run([
            inst!(JUMP 100),
        ].into());
    }
    
    #[test]
    #[should_panic = "jumping to an invalid location"]
    #[cfg(feature = "indirect_jumps")]
    fn jump_negative() {
        Ram::<_, _>::run([
            inst!(LOAD #-2),
            inst!(JUMP @0),
        ].into());
    }
    
    #[test]
    fn jumz_inexistent() {
        Ram::<_, _>::run([
            inst!(LOAD #-2),
            inst!(JUMZ 100),
            inst!(STOP)
        ].into());
    }
}
