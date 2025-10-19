//! Binary operations code generation.

use super::codegen::LlvmCodegen;
use crate::error::{CodegenError, CodegenResult};
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use kivc_ast::BinOp;

impl<'a, 'ctx> LlvmCodegen<'a, 'ctx> {
    /// Builds a binary operation
    pub(super) fn build_binop(
        &self,
        op: &BinOp,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        name: &str,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let left_int = left.into_int_value();
        let right_int = right.into_int_value();

        let result = match op {
            BinOp::Add => self
                .builder
                .build_int_add(left_int, right_int, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?,
            BinOp::Sub => self
                .builder
                .build_int_sub(left_int, right_int, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?,
            BinOp::Mul => self
                .builder
                .build_int_mul(left_int, right_int, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?,
            BinOp::Div => self
                .builder
                .build_int_signed_div(left_int, right_int, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?,
            BinOp::Eq => self
                .builder
                .build_int_compare(IntPredicate::EQ, left_int, right_int, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?,
            BinOp::NotEq => self
                .builder
                .build_int_compare(IntPredicate::NE, left_int, right_int, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?,
            BinOp::Lt => self
                .builder
                .build_int_compare(IntPredicate::SLT, left_int, right_int, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?,
            BinOp::Le => self
                .builder
                .build_int_compare(IntPredicate::SLE, left_int, right_int, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?,
            BinOp::Gt => self
                .builder
                .build_int_compare(IntPredicate::SGT, left_int, right_int, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?,
            BinOp::Ge => self
                .builder
                .build_int_compare(IntPredicate::SGE, left_int, right_int, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?,
        };

        Ok(result.into())
    }
}
