//! Type checking for the Kiv compiler.
//!
//! This crate performs type inference and type checking on HIR,
//! ensuring type safety and reporting type errors.

mod typeck;
mod types;

pub use typeck::typecheck_program;
pub use types::{Type, TypeContext};
