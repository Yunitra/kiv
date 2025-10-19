//! Function code generation.

use super::codegen::LlvmCodegen;
use super::storage::VarStorage;
use crate::error::{CodegenError, CodegenResult};
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;
use kivc_hir::TypeId;
use kivc_mir::MirFunction;
use kivc_typeck::Type;
use std::collections::HashMap;

impl<'a, 'ctx> LlvmCodegen<'a, 'ctx> {
    /// Declares a function signature
    pub(super) fn declare_function(
        &self,
        func: &MirFunction,
        type_table: &HashMap<TypeId, Type>,
    ) -> CodegenResult<FunctionValue<'ctx>> {
        // Map parameter types
        let param_types: Vec<_> = func
            .param_types
            .iter()
            .map(|tid| {
                let ty = type_table.get(tid).unwrap_or(&Type::Unknown);
                crate::types::type_to_metadata(self.context, ty)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let fn_type = if let Some(ret_tid) = func.return_type {
            let ret_ty = type_table.get(&ret_tid).unwrap_or(&Type::Unknown);
            // Get the actual LLVM type and call fn_type on it
            let ty = crate::types::type_to_llvm(self.context, ret_ty)?;
            ty.fn_type(&param_types, false)
        } else {
            self.context.void_type().fn_type(&param_types, false)
        };

        let function = self.module.add_function(&func.name, fn_type, None);

        Ok(function)
    }

    /// Generates the body of a function
    pub(super) fn generate_function(&mut self, func: &MirFunction) -> CodegenResult<()> {
        let function = self
            .module
            .get_function(&func.name)
            .ok_or_else(|| CodegenError::UndefinedFunction(func.name.clone()))?;

        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        // Map variable IDs to storage
        let mut variables: HashMap<kivc_mir::VarId, VarStorage<'ctx>> = HashMap::new();
        let mut cow_vars: Vec<kivc_mir::VarId> = Vec::new(); // Track CoW vars for cleanup

        // Store parameters (assume Copy types for MVP)
        for (i, param_var) in func.params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();
            variables.insert(*param_var, VarStorage::Ssa(param_value));
        }

        // Pre-allocate variables based on their types
        // For MVP: all variables are Int (Copy type), so use SSA
        // In the future, Text variables will use Alloca

        // Generate instructions
        for block in &func.blocks {
            for instr in &block.instructions {
                self.generate_instruction(instr, &mut variables, &mut cow_vars)?;
            }

            // Insert drop calls for CoW variables before terminator
            self.drop_cow_variables(&cow_vars)?;

            self.generate_terminator(&block.terminator, &variables)?;
        }

        Ok(())
    }
}
