//! Value and operand conversion.

use super::codegen::LlvmCodegen;
use super::storage::VarStorage;
use crate::error::{CodegenError, CodegenResult};
use inkwell::values::BasicValueEnum;
use kivc_ast::Literal;
use kivc_mir::{MirOperand, VarId};
use std::collections::HashMap;

impl<'a, 'ctx> LlvmCodegen<'a, 'ctx> {
    /// Converts a MIR operand to an LLVM value
    pub(super) fn operand_to_value(
        &self,
        operand: &MirOperand,
        variables: &HashMap<VarId, VarStorage<'ctx>>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        match operand {
            MirOperand::Var(var_id) => match variables.get(var_id) {
                Some(VarStorage::Ssa(val)) => Ok(*val),
                Some(VarStorage::Alloca(ptr)) => {
                    // Load from alloca for CoW types
                    let load_name = format!("load{}", var_id.as_usize());
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    self.builder
                        .build_load(ptr_type, *ptr, &load_name)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))
                }
                None => Err(CodegenError::General(format!(
                    "Undefined variable: {:?}",
                    var_id
                ))),
            },
            MirOperand::Literal(lit) => self.literal_to_value(lit),
            MirOperand::Unit => Ok(self.context.i64_type().const_zero().into()),
        }
    }

    /// Converts a literal to an LLVM value
    pub(super) fn literal_to_value(&self, lit: &Literal) -> CodegenResult<BasicValueEnum<'ctx>> {
        match lit {
            Literal::Int(val) => Ok(self.context.i64_type().const_int(*val as u64, true).into()),
            Literal::Float(val) => Ok(self.context.f64_type().const_float(*val).into()),
            Literal::Bool(val) => Ok(self
                .context
                .bool_type()
                .const_int(*val as u64, false)
                .into()),
            Literal::Text(s) => {
                // Create a global string constant
                let global_str = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // Call kiv_text_from_cstr to create Text object
                let text_from_cstr =
                    self.module
                        .get_function("kiv_text_from_cstr")
                        .ok_or_else(|| {
                            CodegenError::UndefinedFunction("kiv_text_from_cstr".to_string())
                        })?;

                let text_ptr = self
                    .builder
                    .build_call(
                        text_from_cstr,
                        &[global_str.as_pointer_value().into()],
                        "text",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| {
                        CodegenError::General(
                            "kiv_text_from_cstr should return a value".to_string(),
                        )
                    })?;

                Ok(text_ptr)
            }
            Literal::Unit => Ok(self.context.i64_type().const_zero().into()),
        }
    }
}
