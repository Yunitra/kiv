//! Type checking for the Kiv compiler.
//!
//! This crate performs type inference and type checking on HIR,
//! ensuring type safety and reporting type errors.

mod checker;
mod types;

pub use checker::typecheck_program;
pub use types::{Type, TypeContext};
