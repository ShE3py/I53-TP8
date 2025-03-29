use std::cmp::Ordering;
use crate::model::{Instruction, Integer, Ir, Value};
use crate::optimizer::WoCode;

/// Constant folding.
pub fn fold_consts<T: Integer>(target: &mut WoCode<'_, T>) {
    let mut ir = Ir::default();
    while ir < target.inner.len() {
        let _ir = ir;
        ir = fold_adds(target, ir);
        ir = fold_muls(target, ir);
        ir = fold_divs(target, ir);
        
        if ir == _ir {
            ir += 1;
        }
    }
}

/// Simplifies add/subs; returns where to continue the search.
fn fold_adds<T: Integer>(target: &mut WoCode<'_, T>, ir0: Ir) -> Ir {
    let mut folded = match target.inner.get(ir0) {
        Some(Instruction::Add(Value::Constant(v))) => v,
        Some(Instruction::Sub(Value::Constant(v))) => v.checked_neg().unwrap(),
        _ => return ir0,
    };
    
    let mut ir1 = ir0 + 1;
    if !target.can_combine(ir0, ir1) {
        if folded.is_zero() {
            target.delete_ir(ir0);
        }
        
        return ir1;
    }
    
    loop {
        let rhs = match target.inner.get(ir1) {
            Some(Instruction::Add(Value::Constant(v))) => v,
            Some(Instruction::Sub(Value::Constant(v))) => v.checked_neg().unwrap(),
            Some(Instruction::Mul(Value::Constant(v))) if v == T::one() => T::zero(),
            Some(Instruction::Div(Value::Constant(v))) if v == T::one() => T::zero(),
            _ => break,
        };
        
        target.delete_ir(ir1);
        folded = folded + rhs;
        
        ir1 += 1;
        if !target.can_combine(ir0, ir1) {
            break;
        }
    }
    
    match folded.cmp(&T::zero()) {
        Ordering::Less => target.set_ir(ir0, Instruction::Sub(Value::Constant(folded.checked_neg().unwrap()))),
        Ordering::Equal => target.delete_ir(ir0),
        Ordering::Greater => target.set_ir(ir0, Instruction::Add(Value::Constant(folded))),
    }
    
    ir1
}

/// Simplifies muls; returns where to continue the search.
fn fold_muls<T: Integer>(target: &mut WoCode<'_, T>, ir0: Ir) -> Ir {
    let Some(Instruction::Mul(Value::Constant(mut folded))) = target.inner.get(ir0) else {
        return ir0;
    };
    
    let mut ir1 = ir0 + 1;
    if folded.is_zero() {
        target.set_ir(ir0, Instruction::Load(Value::Constant(T::zero())));
        return ir1;
    }
    
    if !target.can_combine(ir0, ir1) {
        if folded.is_one() {
            target.delete_ir(ir0);
        }
        
        return ir1;
    }
    
    loop {
        let Some(Instruction::Mul(Value::Constant(rhs))) = target.inner.get(ir1) else {
            break;
        };
        
        target.delete_ir(ir1);
        folded = folded * rhs;
        
        ir1 += 1;
        if !target.can_combine(ir0, ir1) {
            break;
        }
    }
    
    if folded.is_one() {
        target.delete_ir(ir0);
    }
    else if folded.is_zero() {
        target.set_ir(ir0, Instruction::Load(Value::Constant(T::zero())));
    }
    else {
        target.set_ir(ir0, Instruction::Mul(Value::Constant(folded)));
    }
    
    ir1
}

// Simplifies divs; returns where to continue the search.
fn fold_divs<T: Integer>(target: &mut WoCode<'_, T>, ir0: Ir) -> Ir {
    let Some(Instruction::Div(Value::Constant(mut folded))) = target.inner.get(ir0) else {
        return ir0;
    };
    
    let mut ir1 = ir0 + 1;
    if !target.can_combine(ir0, ir1) {
        if folded.is_one() {
            target.delete_ir(ir0);
        }
        
        return ir1;
    }
    
    loop {
        let Some(Instruction::Div(Value::Constant(rhs))) = target.inner.get(ir1) else {
            break;
        };
        
        target.delete_ir(ir1);
        folded = folded * rhs;
        
        ir1 += 1;
        if !target.can_combine(ir0, ir1) {
            break;
        }
    }
    
    if folded.is_one() {
        target.delete_ir(ir0);
    }
    else {
        target.set_ir(ir0, Instruction::Div(Value::Constant(folded)));
    }
    
    ir1
}
