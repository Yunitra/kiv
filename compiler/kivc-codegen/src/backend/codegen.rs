//! Main LLVM code generator.

use crate::error::CodegenResult;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use kivc_mir::MirProgram;
use kivc_typeck::Type;

/// LLVM code generator
pub(super) struct LlvmCodegen<'a, 'ctx> {
    pub(super) context: &'ctx Context,
    pub(super) module: &'a Module<'ctx>,
    pub(super) builder: &'a Builder<'ctx>,
}

impl<'a, 'ctx> LlvmCodegen<'a, 'ctx> {
    /// Generates LLVM IR for the entire program
    pub(super) fn generate(&mut self, program: &MirProgram) -> CodegenResult<()> {
        // Forward declare all functions (skip built-ins)
        for func in &program.functions {
            // Skip built-in functions (they're declared in runtime_bindings)
            if func.name == "print" || func.name == "panic" {
                continue;
            }
            self.declare_function(func, &program.type_table)?;
        }

        // Generate all function bodies
        for func in &program.functions {
            // Skip built-in functions
            if func.name == "print" || func.name == "panic" {
                continue;
            }
            self.generate_function(func)?;
        }

        Ok(())
    }

    /// Checks if a type is a Copy type
    #[allow(dead_code)]
    pub(super) fn is_copy_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Int | Type::Float | Type::Bool | Type::Unit)
    }
}
