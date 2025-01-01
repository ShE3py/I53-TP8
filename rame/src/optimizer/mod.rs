#![cfg(not(feature = "indirect_jumps"))]

use crate::model::{Address, Instruction, Integer, Ir, RoCode, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Neg;

fn collect_jump_targets<T: Integer>(code: &RoCode<T>, modified_ir: &HashMap<Ir, Instruction<T>>) -> Vec<Ir> {
    let mut jt = Vec::new();
    for (ir, inst) in code.iter() {
        let inst = modified_ir.get(&ir).copied().unwrap_or(inst);
        
        match inst {
            Instruction::Jump(adr) | Instruction::JumpZero(adr) | Instruction::JumpLtz(adr) | Instruction::JumpGtz(adr) => {
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
    deltas: Vec<(Ir, isize)>,
    /// List of deleted indices.
    deleted_ir: Vec<Ir>,
    /// List of modified indices + new instruction.
    modified_ir: HashMap<Ir, Instruction<T>>,
}

impl<'ro, T: Integer> From<&'ro RoCode<T>> for SeqRewriter<'ro, T> {
    fn from(code: &'ro RoCode<T>) -> Self {
        let modified_ir = HashMap::new();
        let mut deltas = collect_jump_targets(code, &modified_ir).into_iter().map(|jt| (jt, 0)).collect::<Vec<_>>();
        deltas.push((Ir::new(code.len()), 0));
        
        SeqRewriter {
            code,
            deltas,
            deleted_ir: Vec::new(),
            modified_ir,
        }
    }
}

impl<T: Integer> SeqRewriter<'_, T> {
    // DELETION
    
    pub(crate) fn delete_ir(&mut self, ir: Ir) {
        if self.deleted_ir.contains(&ir) {
            return;
        }
        
        assert!(!self.modified_ir.contains_key(&ir));
        
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
        let mut i = 0;
        while i < self.deltas.len() && self.deltas[i].0 < adr {
            i += 1;
        }
        
        adr.checked_add_signed(self.deltas[i].1).expect("integer overflow")
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
    pub(crate) fn can_combine(&self, ir0: Ir, ir1: Ir) -> bool {
        debug_assert!(ir1 > ir0);
        debug_assert!(ir1 <= self.code.len());
        
        for (entry_point, _) in &self.deltas {
            if *entry_point > ir1 {
                return true;
            }
            else if *entry_point > ir0 {
                return false;
            }
        }
        
        unreachable!("a fake entry point should be after the last inst")
    }
    
    /// Returns the next non-deleted IR.
    pub(crate) fn next_ir(&mut self, ir0: Ir) -> Ir {
        self.deleted_ir.sort_unstable();
        
        let mut ir1 = ir0 + 1;
        while self.deleted_ir.binary_search(&ir1).is_ok() {
            ir1 += 1;
        }
        
        ir1
    }
    
    pub(crate) fn set_ir(&mut self, ir: Ir, inst: Instruction<T>) {
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
        rewriter.code.iter()
            .filter(|(ir, _)| rewriter.deleted_ir.binary_search(ir).is_err())
            .map(|(ir, inst)| (ir, rewriter.modified_ir.get(&ir).copied().unwrap_or(inst)))
            .map(|(_, inst)| rewriter.rewrite_inst(inst))
            .collect::<Vec<_>>().as_slice().into()
    }
}

impl<T: Integer> SeqRewriter<'_, T> {
    pub fn remove_nops(&mut self) -> &mut Self {
        self.code.iter().filter(|(_, inst)| *inst == Instruction::Nop).for_each(|(ir, _)| self.delete_ir(ir));
        self
    }
    
    pub fn follow_jumps(&mut self) -> &mut Self {
        let final_adr = |initial_adr: Address| -> Option<Address> {
            let mut path = vec![initial_adr];
            
            let mut adr = initial_adr;
            while let Some(Instruction::Jump(to)) = self.code.get(adr) {
                if path.contains(&to) {
                    return None;
                }
                
                adr = to;
                path.push(adr);
            }
            
            Some(adr).filter(|adr| *adr != initial_adr)
        };
        
        for (ir, inst) in self.code.iter() {
            match inst {
                Instruction::Jump(adr) => {
                    if let Some(adr) = final_adr(adr) {
                        self.set_ir(ir, Instruction::Jump(adr));
                    }
                },
                Instruction::JumpZero(adr) => {
                    if let Some(adr) = final_adr(adr) {
                        self.set_ir(ir, Instruction::JumpZero(adr));
                    }
                },
                Instruction::JumpLtz(adr) => {
                    if let Some(adr) = final_adr(adr) {
                        self.set_ir(ir, Instruction::JumpLtz(adr));
                    }
                },
                Instruction::JumpGtz(adr) => {
                    if let Some(adr) = final_adr(adr) {
                        self.set_ir(ir, Instruction::JumpGtz(adr));
                    }
                },
                _ => {},
            }
        }
        
        self.update_jump_targets();
        self
    }
    
    /// Remove the jump targets that are no longer jumpable into in the modified IR.
    fn update_jump_targets(&mut self) {
        let new_jt = collect_jump_targets(self.code, &self.modified_ir);
        let mut to_rm = Vec::new();
        
        let mut i1 = self.deltas.iter().map(|(entry_point, _)| *entry_point);
        for ep0 in new_jt {
            loop {
                let ep1 = i1.next().unwrap();
                
                if ep0 != ep1 {
                    debug_assert!(ep0 > ep1);
                    to_rm.push(ep0);
                    continue;
                }
                
                break;
            }
        }
        
        to_rm.sort_unstable();
        to_rm.dedup();
        self.deltas.retain_mut(|(entry_point, _)| to_rm.binary_search(entry_point).is_err());
    }
    
    pub fn remove_dead_code(&mut self) -> &mut Self {
        let mut reachable = vec![false; self.code.len()];
        self.find_reachable_code_ir(Ir::default(), &mut reachable);
        
        reachable.into_iter().enumerate().filter_map(|(i, reachable)| (!reachable).then_some(Ir::new(i))).for_each(|ir| self.delete_ir(ir));
        self
    }
    
    /// Mark reachable code starting at the specified instruction into the specified vector.
    fn find_reachable_code_ir(&mut self, mut ir: Ir, reachable: &mut Vec<bool>) {
        if reachable[ir.inner()] {
            return;
        }
        
        while let Some(inst) = self.code.get(ir) {
            reachable[ir.inner()] = true;
            ir += 1;
            
            match inst {
                Instruction::Stop => return,
                Instruction::Jump(adr) => {
                    if reachable[adr.inner()] {
                        return;
                    }
                    
                    ir = adr;
                },
                Instruction::JumpZero(adr) | Instruction::JumpLtz(adr) | Instruction::JumpGtz(adr) => {
                    self.find_reachable_code_ir(adr, reachable);
                },
                _ => {},
            }
        }
        
        panic!("unexpected end of file")
    }
    
    pub fn combine_jumps(&mut self) -> &mut Self {
        for (ir, inst) in self.code.iter() {
            match inst {
                Instruction::Jump(adr) | Instruction::JumpZero(adr) | Instruction::JumpLtz(adr) | Instruction::JumpGtz(adr) => {
                    let next_ir = self.next_ir(ir);
                    
                    if adr == next_ir {
                        // We're jumping to the next inst
                        self.delete_ir(ir);
                    }
                },
                _ => {},
            }
        }
        
        self
    }
}

impl<T: Integer + Neg<Output = T>> SeqRewriter<'_, T> {
    /// Simplifies add/subs; returns where to continue the search.
    fn combine_adds(&mut self, ir0: Ir) -> Ir {
        let mut ir1 = ir0 + 1;
        
        let mut v = match self.code.get(ir0) {
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
            let v1 = match self.code.get(ir1) {
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
    fn combine_muls(&mut self, ir0: Ir) -> Ir {
        let mut ir1 = ir0 + 1;
        
        let Some(Instruction::Mul(Value::Constant(mut v))) = self.code.get(ir0) else {
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
            let Some(Instruction::Mul(Value::Constant(v1))) = self.code.get(ir1) else {
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
    fn combine_divs(&mut self, ir0: Ir) -> Ir {
        let mut ir1 = ir0 + 1;
        
        let Some(Instruction::Div(Value::Constant(mut v))) = self.code.get(ir0) else {
            return ir0;
        };
        
        if !self.can_combine(ir0, ir1) {
            if v.is_one() {
                self.delete_ir(ir0);
            }
            
            return ir1;
        }
        
        loop {
            let Some(Instruction::Div(Value::Constant(v1))) = self.code.get(ir1) else {
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
        let mut ir = Ir::default();
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
        self.remove_nops().follow_jumps().remove_dead_code().combine_jumps().combine_consts()
    }
}

#[cfg(test)]
mod test {
    use crate::inst;
    use super::*;
    
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
        
        let rewriter = SeqRewriter::from(&code);
        assert!(!rewriter.can_combine(Ir::new(1), Ir::new(2)));
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
    
    #[test]
    fn jump_block_combine_consts() {
        let a = RoCode::<i32>::from([
            inst!(ADD #1),
            inst!(ADD #2),
            inst!(ADD #3),
            inst!(ADD #4),
            inst!(JUMP 3),
        ]);
        
        let b = RoCode::<i32>::from([
            inst!(ADD #6),
            inst!(ADD #4),
            inst!(JUMP 1),
        ]);
        
        assert_eq!(SeqRewriter::from(&a).combine_consts().rewritten(), b);
    }
    
    #[test]
    fn follow_jumps() {
        let a = RoCode::<i32>::from([
            inst!(JUMZ 1),
            inst!(JUMP 2),
            inst!(JUMP 3),
            inst!(JUML 4),
        ]);
        
        let b = RoCode::<i32>::from([
            inst!(JUMZ 3),
            inst!(JUMP 3),
            inst!(JUMP 3),
            inst!(JUML 4),
        ]);
        
        assert_eq!(SeqRewriter::from(&a).follow_jumps().rewritten(), b);
    }
    
    #[test]
    fn dont_follow_infinite_jumps() {
        let a = RoCode::<i32>::from([
            inst!(JUMP 0),
            inst!(JUMP 2),
            inst!(JUMP 1),
        ]);
        
        assert_eq!(SeqRewriter::from(&a).follow_jumps().rewritten(), a);
    }
    
    #[test]
    fn remove_dead_code() {
        let a = RoCode::<i32>::from([
            inst!(LOAD #0),
            inst!(JUMP 5),
            inst!(ADD #1),
            inst!(ADD 0),
            inst!(DIV 2),
            inst!(WRITE),
            inst!(JUMP 5),
        ]);
        
        let b = RoCode::<i32>::from([
            inst!(LOAD #0),
            inst!(JUMP 2),
            inst!(WRITE),
            inst!(JUMP 2),
        ]);
        
        assert_eq!(SeqRewriter::from(&a).remove_dead_code().rewritten(), b);
    }
}
