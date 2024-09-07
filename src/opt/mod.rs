use crate::{Address, Instruction, Integer, RoCode};

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
}

impl<'ro, T: Integer> From<&'ro RoCode<T>> for SeqRewriter<'ro, T> {
    fn from(code: &'ro RoCode<T>) -> Self {
        let mut deltas = collect_jump_targets(code).into_iter().map(|jt| (jt, 0)).collect::<Vec<_>>();
        deltas.push((code.len() + 1, 0));
        
        SeqRewriter {
            code,
            deltas,
            deleted_ir: Vec::new(),
        }
    }
}

impl<'ro, T: Integer> SeqRewriter<'ro, T> {
    pub(crate) fn delete_ir(&mut self, ir: usize) {
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
    
    #[must_use]
    pub fn rewritten(&self) -> RoCode<T> {
        self.into()
    }
}

impl<'ro, T: Integer> From<&SeqRewriter<'ro, T>> for RoCode<T> {
    fn from(rewriter: &SeqRewriter<'ro, T>) -> Self {
        rewriter.code.iter().copied().enumerate().filter(|(ir, _)| rewriter.deleted_ir.binary_search(ir).is_err()).map(|(_, inst)| rewriter.rewrite_inst(inst)).collect::<Vec<_>>().as_slice().into()
    }
}

impl<'ro, T: Integer> SeqRewriter<'ro, T> {
    pub fn remove_nops(&mut self) -> &mut Self {
        self.code.iter().enumerate().filter(|(_, inst)| **inst == Instruction::Nop).for_each(|(ir, _)| self.delete_ir(ir));
        self
    }
}

#[cfg(test)]
mod test {
    use crate::{inst, RoCode};
    use crate::opt::SeqRewriter;
    
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
}
