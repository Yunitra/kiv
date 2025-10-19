//! MIR instruction code generation.

use super::codegen::LlvmCodegen;
use super::storage::VarStorage;
use crate::error::{CodegenError, CodegenResult};
use kivc_mir::{MirInstr, VarId};
use std::collections::HashMap;

impl<'a, 'ctx> LlvmCodegen<'a, 'ctx> {
    /// Generates LLVM IR for a single MIR instruction
    pub(super) fn generate_instruction(
        &self,
        instr: &MirInstr,
        variables: &mut HashMap<VarId, VarStorage<'ctx>>,
        _cow_vars: &mut Vec<VarId>,
    ) -> CodegenResult<()> {
        match instr {
            MirInstr::Assign { dest, source } => {
                let value = self.operand_to_value(source, variables)?;

                // For MVP, treat as Copy type (SSA)
                // NOTE: CoW types use simple copy for now (no heap types in current system)
                variables.insert(*dest, VarStorage::Ssa(value));
            }

            MirInstr::BinOp {
                dest,
                op,
                left,
                right,
            } => {
                let left_val = self.operand_to_value(left, variables)?;
                let right_val = self.operand_to_value(right, variables)?;

                let var_name = format!("tmp{}", dest.as_usize());
                let result = self.build_binop(op, left_val, right_val, &var_name)?;

                variables.insert(*dest, VarStorage::Ssa(result));
            }

            MirInstr::Call { dest, func, args } => {
                self.generate_call(dest, func, args, variables)?;
            }

            MirInstr::Nop => {}
        }

        Ok(())
    }

    /// Generates a function call
    fn generate_call(
        &self,
        dest: &Option<VarId>,
        func: &str,
        args: &[kivc_mir::MirOperand],
        variables: &mut HashMap<VarId, VarStorage<'ctx>>,
    ) -> CodegenResult<()> {
        // Handle builtin functions with special behavior
        if func == "print" {
            self.generate_print_call(args, variables)?;
        } else {
            self.generate_regular_call(dest, func, args, variables)?;
        }
        Ok(())
    }

    /// Generates a print() builtin call
    fn generate_print_call(
        &self,
        args: &[kivc_mir::MirOperand],
        variables: &HashMap<VarId, VarStorage<'ctx>>,
    ) -> CodegenResult<()> {
        // print() supports multiple arguments - print each one, then newline
        for (i, arg) in args.iter().enumerate() {
            let arg_value = self.operand_to_value(arg, variables)?;

            // Add space between arguments (except first)
            if i > 0 {
                let space_fn = self
                    .module
                    .get_function("kiv_print_text")
                    .ok_or_else(|| CodegenError::UndefinedFunction("kiv_print_text".to_string()))?;
                let space_str = self
                    .builder
                    .build_global_string_ptr(" ", "space")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                let space_text_fn =
                    self.module
                        .get_function("kiv_text_from_cstr")
                        .ok_or_else(|| {
                            CodegenError::UndefinedFunction("kiv_text_from_cstr".to_string())
                        })?;
                let space_text = self
                    .builder
                    .build_call(
                        space_text_fn,
                        &[space_str.as_pointer_value().into()],
                        "space_text",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .unwrap();
                self.builder
                    .build_call(space_fn, &[space_text.into()], "")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            }

            // Determine print function based on type
            let print_fn_name = if arg_value.is_int_value() {
                "kiv_print_int"
            } else if arg_value.is_float_value() {
                "kiv_print_float"
            } else if arg_value.is_pointer_value() {
                "kiv_print_text"
            } else {
                "kiv_print_int" // fallback
            };

            let print_fn = self
                .module
                .get_function(print_fn_name)
                .ok_or_else(|| CodegenError::UndefinedFunction(print_fn_name.to_string()))?;

            self.builder
                .build_call(print_fn, &[arg_value.into()], "")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Print newline at the end
        let newline_fn = self
            .module
            .get_function("kiv_print_newline")
            .ok_or_else(|| CodegenError::UndefinedFunction("kiv_print_newline".to_string()))?;
        self.builder
            .build_call(newline_fn, &[], "")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(())
    }

    /// Generates a regular function call
    fn generate_regular_call(
        &self,
        dest: &Option<VarId>,
        func: &str,
        args: &[kivc_mir::MirOperand],
        variables: &mut HashMap<VarId, VarStorage<'ctx>>,
    ) -> CodegenResult<()> {
        // Handle other function calls
        let runtime_func_name = match func {
            "panic" => "kiv_panic",
            other => other,
        };

        let function = self.module.get_function(runtime_func_name).ok_or_else(|| {
            CodegenError::UndefinedFunction(format!("{} (mapped from {})", runtime_func_name, func))
        })?;

        let arg_values: Vec<_> = args
            .iter()
            .map(|a| self.operand_to_value(a, variables))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|v| v.into())
            .collect();

        let call_name = if let Some(dest_var) = dest {
            format!("call{}", dest_var.as_usize())
        } else {
            String::new()
        };

        let call_site = self
            .builder
            .build_call(function, &arg_values, &call_name)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        if let Some(dest_var) = dest
            && let Some(return_value) = call_site.try_as_basic_value().left()
        {
            variables.insert(*dest_var, VarStorage::Ssa(return_value));
        }

        Ok(())
    }

    /// Inserts drop calls for CoW variables
    pub(super) fn drop_cow_variables(&self, cow_vars: &[VarId]) -> CodegenResult<()> {
        // For each CoW variable, insert a drop call
        for _var_id in cow_vars {
            // NOTE: CoW optimization deferred until type system includes heap types
            // let drop_fn = self.module.get_function("kiv_text_drop").unwrap();
            // self.builder.build_call(drop_fn, &[var_ptr.into()], "");
        }
        Ok(())
    }
}
