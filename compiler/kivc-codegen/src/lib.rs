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
//! ```no_run
//! # use kivc_codegen::generate_llvm_ir;
//! # use kivc_mir::{MirProgram, MirFunction};
//! # use std::collections::HashMap;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a MIR program
//! let mir_program = MirProgram::new(vec![], HashMap::new());
//! 
//! // Generate LLVM IR
//! let llvm_ir = generate_llvm_ir(&mir_program)?;
//! # Ok(())
//! # }
//! ```

mod backend;
mod error;
mod runtime_bindings;
mod types;

pub use backend::CodegenContext;
pub use error::{CodegenError, CodegenResult};

use inkwell::context::Context;
use kivc_mir::MirProgram;

/// Generates LLVM IR from a MIR program and returns it as a string
pub fn generate_llvm_ir(program: &MirProgram) -> CodegenResult<String> {
    let context = Context::create();
    let codegen_ctx = CodegenContext::new(&context, "kiv_module");

    backend::generate_llvm_ir(&codegen_ctx, program)?;

    let mut ir = codegen_ctx.to_llvm_ir();

    // Clean up: remove empty lines between consecutive declare statements
    ir = ir.replace("\n\ndeclare", "\ndeclare");

    Ok(ir)
}
