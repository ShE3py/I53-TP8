use num_traits::PrimInt;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::num::ParseIntError;
use std::str::FromStr;

mod inst;
mod makro;
mod ro;

pub use inst::{Address, Instruction, Register, Value};
pub use ro::RoCode;

pub trait Integer: PrimInt + Debug + Display + FromStr<Err = ParseIntError> + TryInto<usize, Error: Copy + Clone + Eq + PartialEq + Error + 'static> {
    fn bits() -> u32;
}

impl Integer for u8 { fn bits() -> u32 { 8 } }
impl Integer for u16 { fn bits() -> u32 { 16 } }
impl Integer for u32 { fn bits() -> u32 { 32 } }
impl Integer for u64 { fn bits() -> u32 { 64 } }
impl Integer for u128 { fn bits() -> u32 { 128 } }

impl Integer for i8 { fn bits() -> u32 { 8 } }
impl Integer for i16 { fn bits() -> u32 { 16 } }
impl Integer for i32 { fn bits() -> u32 { 32 } }
impl Integer for i64 { fn bits() -> u32 { 64 } }
impl Integer for i128 { fn bits() -> u32 { 128 } }
