use crate::{Address, Instruction, Integer, RoCode, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Neg;

fn collect_jump_targets<T: Integer>(code: &RoCode<T>) -> Vec<usize> {
    let mut jt = Vec::new();
    for inst in code.iter() {
        match *inst {
            Instruction::Jump(adr) | Instruction::JumpZero(adr) | Instruction::JumpLtz(adr) | Instruction::JumpGtz(adr) => {
                let Address::Constant(adr) = adr;
                jt.push(adr);
            },
            _ => {},
        }
    }
    
    jt.sort_unstable();
    jt.dedup();
    jt
}

#[derive(Debug)]
pub struct SeqRewriter<'ro, T: Integer> {
    code: &'ro RoCode<T>,
    /// First element is the index of a jumped-to instruction, second is the count of instructions added/removed up to that instruction.
    deltas: Vec<(usize, isize)>,
    /// List of deleted indices.
    deleted_ir: Vec<usize>,
    /// List of modified indices + new instruction.
    modified_ir: HashMap<usize, Instruction<T>>,
}

impl<'ro, T: Integer> From<&'ro RoCode<T>> for SeqRewriter<'ro, T> {
    fn from(code: &'ro RoCode<T>) -> Self {
        let mut deltas = collect_jump_targets(code).into_iter().map(|jt| (jt, 0)).collect::<Vec<_>>();
        deltas.push((code.len(), 0));
        
        SeqRewriter {
            code,
            deltas,
            deleted_ir: Vec::new(),
            modified_ir: HashMap::new(),
        }
    }
}

impl<'ro, T: Integer> SeqRewriter<'ro, T> {
    // DELETION
    
    pub(crate) fn delete_ir(&mut self, ir: usize) {
        assert!(!self.deleted_ir.contains(&ir), "duplicated deletion");
        
        for (entry_point, delta) in self.deltas.iter_mut().rev() {
            if *entry_point > ir {
                *delta -= 1;
            } else {
                break;
            }
        }
        
        self.deleted_ir.push(ir);
    }
    
    fn update_adr(&self, adr: Address) -> Address {
        let Address::Constant(adr) = adr;
        
        let mut i = 0;
        while i < self.deltas.len() && self.deltas[i].0 < adr {
            i += 1;
        }
        
        Address::Constant(adr.checked_add_signed(self.deltas[i].1).expect("integer overflow"))
    }
    
    fn rewrite_inst(&self, inst: Instruction<T>) -> Instruction<T> {
        match inst {
            Instruction::Jump(adr) => Instruction::Jump(self.update_adr(adr)),
            Instruction::JumpZero(adr) => Instruction::JumpZero(self.update_adr(adr)),
            Instruction::JumpLtz(adr) => Instruction::JumpLtz(self.update_adr(adr)),
            Instruction::JumpGtz(adr) => Instruction::JumpGtz(self.update_adr(adr)),
            inst => inst,
        }
    }
    
    // MODIFICATION
    
    /// Returns `true` iff there's a jump entry point in `]ir0, ir1]`.
    pub(crate) fn can_combine(&mut self, ir0: usize, ir1: usize) -> bool {
        for (entry_point, _) in &self.deltas {
            if *entry_point > ir1 {
                return true;
            }
            else if *entry_point > ir0 {
                return false;
            }
        }
        
        unreachable!()
    }
    
    pub(crate) fn set_ir(&mut self, ir: usize, inst: Instruction<T>) {
        assert!(!self.deleted_ir.contains(&ir));
        self.modified_ir.insert(ir, inst);
    }
    
    #[must_use]
    pub fn rewritten(&mut self) -> RoCode<T> {
        self.into()
    }
}

impl<'ro, T: Integer> From<&mut SeqRewriter<'ro, T>> for RoCode<T> {
    fn from(rewriter: &mut SeqRewriter<'ro, T>) -> Self {
        rewriter.deleted_ir.sort_unstable();
        rewriter.code.iter().copied().enumerate()
            .filter(|(ir, _)| rewriter.deleted_ir.binary_search(ir).is_err())
            .map(|(ir, inst)| (ir, rewriter.modified_ir.get(&ir).copied().unwrap_or(inst)))
            .map(|(_, inst)| rewriter.rewrite_inst(inst))
            .collect::<Vec<_>>().as_slice().into()
    }
}

impl<'ro, T: Integer> SeqRewriter<'ro, T> {
    pub fn remove_nops(&mut self) -> &mut Self {
        self.code.iter().enumerate().filter(|(_, inst)| **inst == Instruction::Nop).for_each(|(ir, _)| self.delete_ir(ir));
        self
    }
}

impl<'ro, T: Integer + Neg<Output = T>> SeqRewriter<'ro, T> {
    /// Simplifies add/subs; returns where to continue the search.
    fn combine_adds(&mut self, ir0: usize) -> usize {
        let mut ir1 = ir0 + 1;
        
        let mut v = match self.code.get(ir0).copied() {
            Some(Instruction::Add(Value::Constant(v))) => v,
            Some(Instruction::Sub(Value::Constant(v))) => -v,
            _ => return ir0,
        };
        
        if !self.can_combine(ir0, ir1) {
            if v.is_zero() {
                self.delete_ir(ir0);
            }
            
            return ir1;
        }
        
        loop {
            let v1 = match self.code.get(ir1).copied() {
                Some(Instruction::Add(Value::Constant(v))) => v,
                Some(Instruction::Sub(Value::Constant(v))) => -v,
                Some(Instruction::Mul(Value::Constant(v))) if v == T::one() => T::zero(),
                Some(Instruction::Div(Value::Constant(v))) if v == T::one() => T::zero(),
                _ => break,
            };
            
            self.delete_ir(ir1);
            v = v + v1;
            
            ir1 += 1;
            if !self.can_combine(ir0, ir1) {
                break;
            }
        }
        
        match v.cmp(&T::zero()) {
            Ordering::Less => self.set_ir(ir0, Instruction::Sub(Value::Constant(-v))),
            Ordering::Equal => self.delete_ir(ir0),
            Ordering::Greater => self.set_ir(ir0, Instruction::Add(Value::Constant(v))),
        }
        
        ir1
    }
    
    /// Simplifies muls; returns where to continue the search.
    fn combine_muls(&mut self, ir0: usize) -> usize {
        let mut ir1 = ir0 + 1;
        
        let Some(Instruction::Mul(Value::Constant(mut v))) = self.code.get(ir0).copied() else {
            return ir0;
        };
        
        if v.is_zero() {
            self.set_ir(ir0, Instruction::Load(Value::Constant(T::zero())));
            return ir1;
        }
        
        if !self.can_combine(ir0, ir1) {
            if v.is_one() {
                self.delete_ir(ir0);
            }
            
            return ir1;
        }
        
        loop {
            let Some(Instruction::Mul(Value::Constant(v1))) = self.code.get(ir1).copied() else {
                break;
            };
            
            self.delete_ir(ir1);
            v = v * v1;
            
            ir1 += 1;
            if !self.can_combine(ir0, ir1) {
                break;
            }
        }
        
        if v.is_one() {
            self.delete_ir(ir0);
        }
        else if v.is_zero() {
            self.set_ir(ir0, Instruction::Load(Value::Constant(T::zero())));
        }
        else {
            self.set_ir(ir0, Instruction::Mul(Value::Constant(v)));
        }
        
        ir1
    }
    
    // Simplifies divs; returns where to continue the search.
    fn combine_divs(&mut self, ir0: usize) -> usize {
        let mut ir1 = ir0 + 1;
        
        let Some(Instruction::Div(Value::Constant(mut v))) = self.code.get(ir0).copied() else {
            return ir0;
        };
        
        if !self.can_combine(ir0, ir1) {
            if v.is_one() {
                self.delete_ir(ir0);
            }
            
            return ir1;
        }
        
        loop {
            let Some(Instruction::Div(Value::Constant(v1))) = self.code.get(ir1).copied() else {
                break;
            };
            
            self.delete_ir(ir1);
            v = v * v1;
            
            ir1 += 1;
            if !self.can_combine(ir0, ir1) {
                break;
            }
        }
        
        if v.is_one() {
            self.delete_ir(ir0);
        }
        else {
            self.set_ir(ir0, Instruction::Div(Value::Constant(v)));
        }
        
        ir1
    }
    
    pub fn combine_consts(&mut self) -> &mut Self {
        let mut ir = 0;
        while ir < self.code.len() {
            let _ir = ir;
            ir = self.combine_adds(ir);
            ir = self.combine_muls(ir);
            ir = self.combine_divs(ir);
            
            if ir == _ir {
                ir += 1;
            }
        }
        
        self
    }
    
    pub fn optimize(&mut self) -> &mut Self {
        self.remove_nops().combine_consts()
    }
}

#[cfg(test)]
mod test {
    use crate::opt::SeqRewriter;
    use crate::{inst, RoCode};
    
    #[test]
    fn remove_nops() {
        let a = RoCode::<i32>::from([
            inst!(NOP),
            inst!(JUMP 3),
            inst!(NOP),
            inst!(WRITE),
            inst!(NOP),
            inst!(NOP),
            inst!(JUMP 3),
        ]);
        
        let b = RoCode::<i32>::from([
            inst!(JUMP 1),
            inst!(WRITE),
            inst!(JUMP 1),
        ]);
        
        assert_eq!(SeqRewriter::from(&a).remove_nops().rewritten(), b);
    }
    
    #[test]
    fn cant_combine() {
        let code = RoCode::<i32>::from([
            inst!(JUMP 2),
            inst!(ADD #1),
            inst!(ADD #3),
        ]);
        
        let mut rewriter = SeqRewriter::from(&code);
        assert!(!rewriter.can_combine(1, 2));
    }
    
    #[test]
    fn remove_identities() {
        let a = RoCode::<i32>::from([
            inst!(ADD #0),
            inst!(SUB #0),
            inst!(MUL #1),
            inst!(DIV #1),
        ]);
        
        let b = RoCode::<i32>::from([]);
        
        assert_eq!(SeqRewriter::from(&a).combine_consts().rewritten(), b);
    }
    
    #[test]
    fn combine_consts() {
        let a = RoCode::<i32>::from([
            inst!(ADD #0),
            inst!(SUB #-1),
            inst!(ADD #2),
            inst!(MUL #1),
            inst!(DIV #2),
            inst!(DIV #3),
        ]);
        
        let b = RoCode::<i32>::from([
            inst!(ADD #3),
            inst!(DIV #6),
        ]);
        
        assert_eq!(SeqRewriter::from(&a).combine_consts().rewritten(), b);
    }
}
