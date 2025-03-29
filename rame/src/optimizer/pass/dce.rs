//! Pass: Dead Code Elimination

use crate::model::{Instruction, Integer, Ir};
use crate::optimizer::WoCode;

/// Remove instructions that are jumped over.
pub fn remove_unreachable_code<T: Integer>(target: &mut WoCode<'_, T>) {
    let mut reachable = vec![false; target.inner.len()];
    find_reachable_code_ir(target, Ir::default(), &mut reachable);
    
    for (i, reachable) in reachable.iter().enumerate() {
        if !reachable {
            target.delete_ir(Ir::new(i));
        }
    }
}

/// Mark reachable code starting at the specified instruction into the specified vector.
fn find_reachable_code_ir<T: Integer>(target: &WoCode<'_, T>, mut ir: Ir, reachable: &mut Vec<bool>) {
    if reachable[ir.inner()] {
        return;
    }
    
    while let Some(inst) = target.inner.get(ir) {
        reachable[ir.inner()] = true;
        ir += 1;
        
        if let Some(adr) = inst.jump() {
            find_reachable_code_ir(target, adr, reachable);
        }
        
        if matches!(inst, Instruction::Stop | Instruction::Jump(_)) {
            return;
        }
    }
}
