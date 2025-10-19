//! LLVM IR generation backend.
//!
//! This module is organized into focused sub-modules for maintainability:
//! - `context`: Code generation context management
//! - `codegen`: Main code generator
//! - `function`: Function declaration and generation
//! - `instruction`: MIR instruction code generation
//! - `terminator`: Terminator instruction generation
//! - `value`: Value and operand conversion
//! - `binop`: Binary operations
//! - `storage`: Variable storage strategies

mod binop;
mod codegen;
mod context;
mod function;
mod instruction;
mod storage;
mod terminator;
mod value;

pub use context::CodegenContext;

use crate::error::CodegenResult;
use codegen::LlvmCodegen;
use kivc_mir::MirProgram;

/// Generates LLVM IR from a MIR program
pub fn generate_llvm_ir<'ctx>(
    ctx: &CodegenContext<'ctx>,
    program: &MirProgram,
) -> CodegenResult<()> {
    let mut codegen = LlvmCodegen {
        context: ctx.context,
        module: &ctx.module,
        builder: &ctx.builder,
    };

    codegen.generate(program)
}
