//! # Code Generation for Kiv
//!
//! Generates target code from MIR (Mid-level Intermediate Representation).
//! By default, a lightweight textual backend is used that emits a stable,
//! human-readable representation of MIR suitable for debugging and tooling.
//! When the `llvm` feature is enabled, the LLVM backend (via `inkwell`) is used
//! to produce LLVM IR.

mod error;

#[cfg(feature = "llvm")]
mod llvm_backend;
#[cfg(feature = "llvm")]
mod runtime_bindings;
#[cfg(feature = "llvm")]
mod types;

mod textual_backend;

pub use error::{CodegenError, CodegenResult};
#[cfg(feature = "llvm")]
pub use llvm_backend::CodegenContext;

use kivc_mir::MirProgram;

/// Generates LLVM IR from a MIR program and returns it as a string
pub fn generate_llvm_ir(program: &MirProgram) -> CodegenResult<String> {
    #[cfg(feature = "llvm")]
    {
        use inkwell::context::Context;
        let context = Context::create();
        let codegen_ctx = CodegenContext::new(&context, "kiv_module");
        llvm_backend::generate_llvm_ir(&codegen_ctx, program)?;
        let mut ir = codegen_ctx.to_llvm_ir();
        ir = ir.replace("\n\ndeclare", "\ndeclare");
        return Ok(ir);
    }

    // Fallback textual backend (no LLVM required)
    Ok(textual_backend::emit_textual_ir(program))
}
