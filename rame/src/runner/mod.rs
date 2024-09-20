use crate::error::{format_err, format_help, RunError};
use crate::model::{Instruction, Integer, Register, RoCode, Value};
use crate::runner::mem::{Loc, LocEntry};
use std::cell::{Cell, UnsafeCell};
use std::iter::{self, Fuse};
use std::process::exit;

pub mod mem;

type Memory<T> = UnsafeCell<Vec<Cell<Loc<T>>>>;

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
    ir: usize,
}

impl<T: Integer, I: Iterator<Item = T>> Ram<T, I> {
    pub fn new(code: RoCode<T>, input: impl IntoIterator<IntoIter = I>) -> Ram<T, I> {
        let Some(inst) = code.first().copied() else {
            if cfg!(test) {
                panic!("empty instruction table");
            }
            else {
                eprintln!("error: empty instruction table");
                exit(1);
            }
        };
        
        Ram {
            input: input.into_iter().fuse(),
            output: vec![],
            memory: Memory::default(),
            code,
            inst,
            ir: 0,
        }
    }
    
    /// Executes the next instruction.
    pub fn step(&mut self) -> Result<bool /* continue */, RunError<T>> {
        self.ir += 1;
        
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
                self.binop(v, |a, b| Some(a.rem(*b)))?;
            },
            Instruction::Jump(addr) => {
                self.ir = addr.get(self)?;
            }
            Instruction::JumpZero(addr) => {
                if self.acc().get()?.is_zero() {
                    self.ir = addr.get(self)?;
                }
            },
            Instruction::JumpLtz(addr) => {
                if self.acc().get()? < T::zero() {
                    self.ir = addr.get(self)?;
                }
            },
            Instruction::JumpGtz(addr) => {
                if self.acc().get()? > T::zero() {
                    self.ir = addr.get(self)?;
                }
            },
            Instruction::Stop => {
                return Ok(false);
            }
            Instruction::Nop => {}
        }
        
        match self.code.get(self.ir).copied() {
            Some(inst) => self.inst = inst,
            None => return Err(RunError::Eof),
        }
        
        Ok(true)
    }
    
    /// Either `INC` or `DEC`.
    fn unop<F: Fn(&T, &T) -> Option<T>>(&self, reg: Register, f: F) -> Result<(), RunError<T>> {
        let loc = reg.loc(self)?;
        let v = loc.get()?;
        f(&v, &T::one()).map(|r| loc.set(r)).ok_or(RunError::IntegerOverfow)
    }
    
    /// Arithmetic instructions on ACC.
    fn binop<F: Fn(&T, &T) -> Option<T>>(&mut self, v: Value<T>, f: F) -> Result<(), RunError<T>> {
        let acc = self.acc().get()?;
        let v = v.get(self)?;
        f(&acc, &v).map(|r| self.acc().set(r)).ok_or(RunError::IntegerOverfow)
    }
    
    /// Runs the whole program, and returns its output.
    pub fn run(mut self) -> Vec<T> {
        loop {
            let ir = self.ir;
            
            match self.step() {
                Ok(true) => continue,
                Ok(false) => break self.output,
                Err(e) => self.emit_err(ir, e),
            }
        }
    }
    
    #[expect(clippy::needless_pass_by_value)]
    fn emit_err(&self, ir: usize, e: RunError<T>) -> ! {
        let path = "anon".as_ref();
        let inst = self.code.get(ir).copied();
        let snip = inst.map(|inst| inst.to_string()).unwrap_or_default();
        let mut err = format_err(path, &snip, ir, e.to_string());
        
        if !matches!(e, RunError::Eof | RunError::ReadUninit { .. }) {
            if let Some(inst) = inst {
                // Show ACC value
                if inst.should_print_acc() {
                    err.push('\n');
                    err.push_str(&format_help(path, ir, format!("ACC = {}", self.acc().inner.get())));
                }
                
                // Show register value
                match inst.register() {
                    Some(Register::Direct(adr)) if !matches!(e, RunError::ReadUninit { .. }) => {
                        let loc = self.loc(adr);
                        
                        err.push('\n');
                        err.push_str(&format_help(path, ir, loc));
                    },
                    Some(Register::Indirect(adr)) => {
                        let val = self.loc(adr).get().unwrap();
                        let loc = match val.try_into() {
                            Ok(adr) => Ok(self.loc(adr)),
                            Err(err) => Err(RunError::InvalidAddress { adr: val, err }),
                        };
                        
                        let msg = loc.map_or_else(|err| format!("<{err}>"), |val| val.to_string());
                        
                        err.push('\n');
                        err.push_str(&format_help(path, ir, msg));
                        
                    },
                    _ => {},
                }
            }
        }
        
        match e {
            RunError::IntegerOverfow => {
                err.push('\n');
                err.push_str(&format_help(path, ir, format!("using `--bits={}`; only values from {} to {} are accepted.", size_of::<T>() * 8, &T::min_value(), &T::max_value())));
            },
            RunError::Eof => {
                err.push('\n');
                err.push_str(&format_help(path, ir, "missing `STOP`?"));
            },
            _ => {}
        }
        
        if cfg!(test) { panic!("{err}") } else { eprintln!("{err}") };
        exit(1);
    }
    
    pub fn output(&self) -> &[T] {
        &self.output
    }
    
    fn loc(&self, adr: usize) -> LocEntry<'_, T> {
        // SAFETY: this function is the only one that uses `self.memory`,
        //  we don't call code that could call this function a 2nd time,
        //  so no there's references that point to our memory.
        let memory = unsafe { &mut *self.memory.get() };
        
        if adr >= memory.len() {
            memory.resize(adr + 1, Cell::new(Loc::Uninit));
        };
        
        LocEntry {
            adr,
            inner: &memory[adr],
        }
    }
    
    fn acc(&self) -> LocEntry<'_, T> {
        self.loc(0)
    }
    
    pub const fn code(&self) -> &RoCode<T> {
        &self.code
    }
    
    pub const fn ir(&self) -> usize {
        self.ir
    }
}

impl<T: Integer, I: Iterator<Item = T> + Default> Default for Ram<T, I> {
    fn default() -> Self {
        Ram {
            input: I::default().fuse(),
            output: vec![],
            memory: Memory::default(),
            code: RoCode::default(),
            inst: Instruction::Stop,
            ir: 0,
        }
    }
}

impl<T: Integer> Ram<T, iter::Empty<T>> {
    pub fn empty() -> Self {
        Self::default()
    }
    
    pub fn without_inputs(code: RoCode<T>) -> Self {
        Self::new(code, iter::empty())
    }
}

impl<T: Integer> From<RoCode<T>> for Ram<T, iter::Empty<T>> {
    fn from(code: RoCode<T>) -> Self {
        Self::without_inputs(code)
    }
}

impl<T: Integer> From<&[Instruction<T>]> for Ram<T, iter::Empty<T>> {
    fn from(code: &[Instruction<T>]) -> Self {
        Self::without_inputs(code.into())
    }
}

impl<T: Integer, const N: usize> From<[Instruction<T>; N]> for Ram<T, iter::Empty<T>> {
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
    #[should_panic = "empty instruction table"]
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
    #[should_panic = "jumping to an inexistent location"]
    #[cfg(feature = "dynamic_jumps")]
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
