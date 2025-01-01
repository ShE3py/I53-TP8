#[cfg(all(feature = "optimizer", feature = "indirect_jumps"))]
compile_error!("the `optimizer` feature is not compatible with the `indirect_jumps` feature");

pub(crate) mod error;
pub mod model;
pub mod runner;

#[cfg(feature = "optimizer")]
pub mod optimizer;
