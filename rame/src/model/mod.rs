//! The implemtation model of the [random-access machines.](https://en.wikipedia.org/wiki/Random-access_machine)

use num_traits::PrimInt;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::str::FromStr;

mod error;
mod inst;
mod ir;
mod makro;
mod ro;

pub use error::ParseInstructionError;
pub use inst::{Address, Instruction, Register, Value};
pub use ir::Ir;
pub use ro::RoCode;

/// The value type this model can works on.
pub trait Integer: PrimInt
    + Debug + Display
    + FromStr<Err: Error + 'static>
    + TryInto<usize, Error: Error + 'static> {}

impl<T: PrimInt
    + Debug + Display
    + FromStr<Err: Error + 'static>
    + TryInto<usize, Error: Error + 'static>> Integer for T {}
