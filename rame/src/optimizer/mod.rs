#![cfg(not(feature = "indirect_jumps"))]

mod rw;

use crate::model::{Integer, RoCode};

pub use rw::WoCode;

pub type Pass<T> = fn(&mut WoCode<'_, T>);

/// Run the specified optimization pass on the specified code.
pub fn run_pass<T: Integer>(target: &'_ RoCode<T>, pass: Pass<T>) -> RoCode<T> {
    let mut target = WoCode::from(target);
    pass(&mut target);
    (&target).into()
}

/// Run all optimization passes on the specified code.
pub fn run_passes<T: Integer>(target: &'_ RoCode<T>) -> RoCode<T> {
    // FIXME: avoid copying
    let target = run_pass( target, pass::remove_nops);
    let target = run_pass(&target, pass::simplify_jumps);
    let target = run_pass(&target, pass::remove_unreachable_code);
    let target = run_pass(&target, pass::fold_consts);
    target
}

pub mod pass {
    use crate::model::{Address, Instruction, Integer};
    use crate::optimizer::WoCode;
    
    mod dce;
    mod fold;
    
    pub use dce::remove_unreachable_code;
    pub use fold::fold_consts;
    
    /// Remove all [`Instruction::Nop`].
    pub fn remove_nops<T: Integer>(target: &mut WoCode<'_, T>) {
        for (ir, inst) in target.inner.enumerate() {
            if inst == Instruction::Nop {
                target.delete_ir(ir);
            }
        }
    }
    
    /// Simplify jumps by following unconditional ones.
    pub fn simplify_jumps<T: Integer>(target: &mut WoCode<'_, T>) {
        // Follow jumps, returning an unconditional jump target.
        let final_adr = |initial_adr: Address| -> Option<Address> {
            let mut path = vec![initial_adr];
            
            let mut adr = initial_adr;
            while let Some(Instruction::Jump(to)) = target.inner.get(adr) {
                if path.contains(&to) {
                    // infinite loop detected!
                    return None;
                }
                
                adr = to;
                path.push(adr);
            }
            
            Some(adr).filter(|adr| *adr != initial_adr)
        };
        
        for (ir, inst) in target.inner.enumerate() {
            if let Some(adr) = inst.jump().and_then(final_adr) {
                target.set_ir(ir, inst.map_adr(|_| adr));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::inst;
    use crate::model::Ir;
    
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
        
        assert_eq!(run_pass(&a, pass::remove_nops), b);
    }
    
    #[test]
    fn cant_combine() {
        let code = RoCode::<i32>::from([
            inst!(JUMP 2),
            inst!(ADD #1),
            inst!(ADD #3),
        ]);
        
        let rewriter = WoCode::from(&code);
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
        
        assert_eq!(run_pass(&a, pass::fold_consts), b);
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
        
        assert_eq!(run_pass(&a, pass::fold_consts), b);
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
        
        assert_eq!(run_pass(&a, pass::fold_consts), b);
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
        
        assert_eq!(run_pass(&a, pass::simplify_jumps), b);
    }
    
    #[test]
    fn dont_follow_infinite_jumps() {
        let a = RoCode::<i32>::from([
            inst!(JUMP 0),
            inst!(JUMP 2),
            inst!(JUMP 1),
        ]);
        
        assert_eq!(run_pass(&a, pass::simplify_jumps), a);
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
        
        assert_eq!(run_pass(&a, pass::remove_unreachable_code), b);
    }
}
