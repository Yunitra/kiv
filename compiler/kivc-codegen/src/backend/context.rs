//! Code generation context management.

use crate::error::{CodegenError, CodegenResult};
use crate::runtime_bindings::declare_runtime_functions;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;

/// Code generation context
pub struct CodegenContext<'ctx> {
    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) builder: Builder<'ctx>,
}

impl<'ctx> CodegenContext<'ctx> {
    /// Creates a new codegen context
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        // Declare runtime functions
        declare_runtime_functions(context, &module);

        Self {
            context,
            module,
            builder,
        }
    }

    /// Returns the LLVM IR as a string
    pub fn to_llvm_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Writes the LLVM IR to a file
    pub fn write_to_file(&self, path: &std::path::Path) -> CodegenResult<()> {
        self.module
            .print_to_file(path)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }
}
