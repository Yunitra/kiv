//! Terminator instruction code generation.

use super::codegen::LlvmCodegen;
use super::storage::VarStorage;
use crate::error::{CodegenError, CodegenResult};
use kivc_mir::{MirTerminator, VarId};
use std::collections::HashMap;

impl<'a, 'ctx> LlvmCodegen<'a, 'ctx> {
    /// Generates a terminator instruction
    pub(super) fn generate_terminator(
        &self,
        terminator: &MirTerminator,
        variables: &HashMap<VarId, VarStorage<'ctx>>,
    ) -> CodegenResult<()> {
        match terminator {
            MirTerminator::Return { value } => {
                if let Some(op) = value {
                    let val = self.operand_to_value(op, variables)?;
                    self.builder
                        .build_return(Some(&val))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                } else {
                    self.builder
                        .build_return(None)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }
            }
            MirTerminator::Unreachable => {
                self.builder
                    .build_unreachable()
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            }
            MirTerminator::Jump { .. } | MirTerminator::Branch { .. } => {
                // For MVP, we only support single basic block functions
                return Err(CodegenError::General(
                    "Multi-block control flow not yet supported".to_string(),
                ));
            }
        }
        Ok(())
    }
}
