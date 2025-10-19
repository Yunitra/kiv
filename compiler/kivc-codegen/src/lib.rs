//! # Code Generation for Kiv
//!
//! This module generates LLVM IR from MIR (Mid-level Intermediate Representation).
//! It uses the `inkwell` library to interface with LLVM 18.
//!
//! ## Architecture
//!
//! - **LLVM Backend**: Generates LLVM IR from MIR using inkwell
//! - **Type Mapping**: Maps Kiv types to LLVM types
//! - **Runtime Bindings**: Declares runtime functions for linking
//!
//! ## Example
//!
//! ```ignore
//! use kivc_codegen::{CodegenContext, generate_llvm_ir};
//! use kivc_mir::MirProgram;
//!
//! let context = CodegenContext::new("my_module");
//! let llvm_ir = generate_llvm_ir(&context, &mir_program)?;
//! ```

mod error;
mod llvm_backend;
mod runtime_bindings;
mod types;

pub use error::{CodegenError, CodegenResult};
pub use llvm_backend::CodegenContext;

use inkwell::context::Context;
use kivc_mir::MirProgram;

/// Generates LLVM IR from a MIR program and returns it as a string
pub fn generate_llvm_ir(program: &MirProgram) -> CodegenResult<String> {
    let context = Context::create();
    let codegen_ctx = CodegenContext::new(&context, "kiv_module");

    llvm_backend::generate_llvm_ir(&codegen_ctx, program)?;

    let mut ir = codegen_ctx.to_llvm_ir();

    // Clean up: remove empty lines between consecutive declare statements
    ir = ir.replace("\n\ndeclare", "\ndeclare");

    Ok(ir)
}
