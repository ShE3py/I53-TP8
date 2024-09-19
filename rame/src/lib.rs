pub mod error;
pub mod model;
pub mod runner;

#[cfg(all(feature = "optimizer", not(feature = "dynamic_jumps")))]
pub mod optimizer;

#[cfg(all(feature = "optimizer", feature = "dynamic_jumps"))]
compile_error!("the `optimizer` feature is not compatible with the `dyanmic_jumps` feature");
