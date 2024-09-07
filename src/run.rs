use crate::error::{format_err, format_help, RunError};
use crate::{Address, Instruction, Integer, Register, RoCode, Value};
use std::fmt::{Display, Formatter};
use std::iter::Fuse;
use std::process::exit;
use std::{fmt, iter};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Loc<T: Integer> {
    #[default] Uninit,
    Init(T)
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct LocEntry<'a, T: Integer> {
    pub adr: usize,
    pub inner: &'a mut Loc<T>,
}

#[derive(Debug)]
#[must_use]
pub struct Ram<T: Integer, I: Iterator<Item = T>> {
    input: Fuse<I>,
    output: Vec<T>,
    memory: Vec<Loc<T>>,
    code: RoCode<T>,
    /// The next instruction to run.
    inst: Instruction<T>,
    /// Instruction register (the index of `inst`).
    ir: usize,
}

impl<T: Integer, I: Iterator<Item = T>> Ram<T, I> {
    pub fn new(code: RoCode<T>, input: impl IntoIterator<IntoIter = I>) -> Ram<T, I> {
        let inst = code.first().copied().expect("empty instruction table");
        
        Ram {
            input: input.into_iter().fuse(),
            output: vec![],
            memory: vec![],
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
    fn unop<F: Fn(&T, &T) -> Option<T>>(&mut self, reg: Register, f: F) -> Result<(), RunError<T>> {
        let mut loc = reg.loc(self)?;
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
    
    fn emit_err(&mut self, ir: usize, e: RunError<T>) -> ! {
        let path = "anon".as_ref();
        let inst = self.code.get(ir).copied();
        let snip = inst.map(|inst| inst.to_string()).unwrap_or_default();
        let mut err = format_err(path, &snip, ir, e.to_string());
        
        if let Some(inst) = inst {
            // Show ACC value
            if inst.should_print_acc() && e != RunError::Eof {
                err.push('\n');
                err.push_str(&format_help(path, ir, format!("ACC = {}", self.acc().inner)));
            }
            
            // Show register value
            if e != RunError::Eof {
                match inst.register() {
                    Some(Register::Direct(adr)) if !matches!(e, RunError::ReadUninit { .. }) => {
                        let loc = self.loc(adr);
                    
                        err.push('\n');
                        err.push_str(&format_help(path, ir, loc));
                    },
                    Some(Register::Indirect(adr)) => {
                        match self.loc(adr).get() {
                            Ok(val) => {
                                let loc = match val.try_into() {
                                    Ok(adr) => Ok(self.loc(adr)),
                                    Err(err) => Err(RunError::InvalidAddress { adr: val, err }),
                                };
                                
                                let msg = loc.map_or_else(|err| format!("<{err}>"), |val| val.to_string());
                                
                                err.push('\n');
                                err.push_str(&format_help(path, ir, msg));
                            },
                            Err(e) => {
                                assert!(matches!(e, RunError::ReadUninit { .. }));
                            }
                        }
                    },
                    _ => {},
                }
            }
        }
        
        match e {
            RunError::IntegerOverfow => {
                err.push('\n');
                err.push_str(&format_help(path, ir, format!("using `--bits={}`; only values from {} to {} are accepted.", &T::bits(), &T::min_value(), &T::max_value())));
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
    
    fn loc(&mut self, adr: usize) -> LocEntry<'_, T> {
        if adr >= self.memory.len() {
            self.memory.resize(adr + 1, Loc::Uninit);
        };
        
        LocEntry {
            adr,
            inner: &mut self.memory[adr],
        }
    }
    
    fn acc(&mut self) -> LocEntry<'_, T> {
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
            memory: vec![],
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

impl<T: Integer> Loc<T> {
    pub fn set(&mut self, v: T) {
        *self = Loc::Init(v);
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

impl<T: Integer> LocEntry<'_, T> {
    pub fn set(&mut self, v: T) {
        self.inner.set(v);
    }
    
    pub fn get(&mut self) -> Result<T, RunError<T>> {
        match self.inner {
            Loc::Uninit => Err(RunError::ReadUninit { adr: self.adr }),
            Loc::Init(v) => Ok(*v),
        }
    }
}

impl<T: Integer> Display for LocEntry<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "R{} = {}", self.adr, self.inner)
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
    pub fn loc<'ram, T: Integer, I: Iterator<Item = T>>(&self, ram: &'ram mut Ram<T, I>) -> Result<LocEntry<'ram, T>, RunError<T>> {
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
    pub fn get<T: Integer, I: Iterator<Item = T>>(&self, ram: &mut Ram<T, I>) -> Result<usize, RunError<T>> {
        let ir = match *self {
            Address::Constant(adr) => adr,
            Address::Register(adr) => {
                let adr = ram.loc(adr).get()?;
                
                #[allow(clippy::map_err_ignore)]
                adr.try_into().map_err(|_| RunError::InexistentJump)?
            }
        };
        
        if ir >= ram.code.len() {
            return Err(RunError::InexistentJump);
        }
        
        Ok(ir)
    }
}

#[cfg(test)]
mod test {
    use crate::run::Ram;
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
        Ram::<_, _>::run([
            inst!(LOAD #100),
            inst!(JUMP @0),
        ].into());
    }
    
    #[test]
    #[should_panic = "jumping to an inexistent location"]
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
            inst!(JUMZ @0),
            inst!(STOP)
        ].into());
    }
}
