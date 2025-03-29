use std::collections::HashMap;
use crate::model::{Address, Instruction, Integer, Ir, RoCode};

/// Represents a write-only code segment.
#[derive(Debug)]
pub struct WoCode<'ro, T: Integer> {
    /// All the IRs are for this code.
    pub(super) inner: &'ro RoCode<T>,
    
    /// First element is the index of a jumped-to instruction, second is the count of instructions added/removed up to that instruction.
    /// This vector is sorted by `Ir`.
    deltas: Vec<(Address, isize)>,
    
    /// List of deleted IRs.
    deleted_ir: Vec<Ir>,
    
    /// List of modified IRs + their new instruction.
    modified_ir: HashMap<Ir, Instruction<T>>,
}

impl<'ro, T: Integer> From<&'ro RoCode<T>> for WoCode<'ro, T> {
    fn from(target: &'ro RoCode<T>) -> Self {
        let mut deltas = target.iter().filter_map(Instruction::jump).map(|entrypoint| (entrypoint, 0)).collect::<Vec<_>>();
        deltas.sort_unstable();
        deltas.push((Ir::new(target.len()), 0));
        deltas.dedup();
        
        WoCode {
            inner: target,
            deltas,
            deleted_ir: Vec::new(),
            modified_ir: HashMap::new(),
        }
    }
}

impl<T: Integer> WoCode<'_, T> {
    /// Mark the specified [`Ir`] as deleted.
    pub fn delete_ir(&mut self, ir: Ir) {
        assert!(!self.modified_ir.contains_key(&ir));
        
        if let Err(i) = self.deleted_ir.binary_search(&ir) {
            self.deleted_ir.insert(i, ir);
        }
        
        // Shift instructions left
        for (entry_point, delta) in self.deltas.iter_mut().rev() {
            if *entry_point > ir {
                *delta -= 1;
            } else {
                break;
            }
        }
    }
    
    /// Edit the specified [`Ir`]'s [`Instruction`].
    pub fn set_ir(&mut self, ir: Ir, inst: Instruction<T>) {
        assert!(self.deleted_ir.binary_search(&ir).is_err());
        self.modified_ir.insert(ir, inst);
    }
    
    /// Returns the next non-deleted IR.
    pub fn next_ir(&mut self, ir0: Ir) -> Ir {
        match self.deleted_ir.binary_search(&(ir0 + 1)) {
            Err(_) => ir0 + 1,
            Ok(i) => {
                let mut d = 1;
                loop {
                    d += 1;
                    if self.deleted_ir[i + d] > ir0 + d {
                        break ir0 + d
                    }
                }
            }
        }
    }
    
    /// Returns `true` iff there's no jump entrypoints in `]ir0, ir1]`.
    pub fn can_combine(&self, ir0: Ir, ir1: Ir) -> bool {
        debug_assert!(ir1 > ir0);
        debug_assert!(ir1 <= self.inner.len());
        
        for (entrypoint, _) in self.deltas.iter().copied() {
            if entrypoint > ir1 {
                // the next jump is after ir1
                return true;
            }
            else if entrypoint > ir0 {
                return false;
            }
        }
        
        unreachable!("a fake entry point should be after the last inst")
    }
}

impl<T: Integer> Instruction<T> {
    /// Map all jump targets.
    pub(super) fn map_adr<F: Fn(Address) -> Address>(self, f: F) -> Instruction<T> {
        match self {
            Instruction::Jump(adr) => Instruction::Jump(f(adr)),
            Instruction::JumpZero(adr) => Instruction::JumpZero(f(adr)),
            Instruction::JumpLtz(adr) => Instruction::JumpLtz(f(adr)),
            Instruction::JumpGtz(adr) => Instruction::JumpGtz(f(adr)),
            _ => self,
        }
    }
}

impl<'ro, T: Integer> From<&WoCode<'ro, T>> for RoCode<T> {
    fn from(code: &WoCode<'ro, T>) -> Self {
        /// Rewrite an jump address.
        fn update_adr(deltas: &Vec<(Ir, isize)>, adr: Address) -> Address {
            let mut delta = 0;
            while delta < deltas.len() && deltas[delta].0 < adr {
                delta += 1;
            }
            
            adr.checked_add_signed(deltas[delta].1).expect("integer overflow")
        }
        
        code.inner.enumerate()
            // delete
            .filter(|(ir, _)| code.deleted_ir.binary_search(ir).is_err())
            // edit
            .map(|(ir, inst)| (ir, code.modified_ir.get(&ir).copied().unwrap_or(inst)))
            // rewrite adrs
            .map(|(_, inst)| inst.map_adr(|adr| update_adr(&code.deltas, adr)))
            // collect
            .collect::<Vec<_>>().as_slice().into()
    }
}
