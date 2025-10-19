//! LLVM IR generation backend using inkwell.

use crate::error::{CodegenError, CodegenResult};
use crate::runtime_bindings::declare_runtime_functions;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::IntPredicate;
use kivc_ast::{BinOp, Literal};
use kivc_hir::TypeId;
use kivc_mir::{MirFunction, MirInstr, MirOperand, MirProgram, MirTerminator, VarId};
use kivc_typeck::Type;
use std::collections::HashMap;

/// Code generation context
pub struct CodegenContext<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
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

struct LlvmCodegen<'a, 'ctx> {
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    builder: &'a Builder<'ctx>,
}

/// Variable storage strategy
enum VarStorage<'ctx> {
    /// SSA value (for Copy types: Int, Float, Bool)
    Ssa(BasicValueEnum<'ctx>),
    /// Stack allocation (for CoW types: Text)
    #[allow(dead_code)] // Reserved for future CoW type support
    Alloca(PointerValue<'ctx>),
}

impl<'a, 'ctx> LlvmCodegen<'a, 'ctx> {
    fn generate(&mut self, program: &MirProgram) -> CodegenResult<()> {
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

    fn declare_function(
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

    fn generate_function(&mut self, func: &MirFunction) -> CodegenResult<()> {
        let function = self
            .module
            .get_function(&func.name)
            .ok_or_else(|| CodegenError::UndefinedFunction(func.name.clone()))?;

        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        // Map variable IDs to storage
        let mut variables: HashMap<VarId, VarStorage<'ctx>> = HashMap::new();
        let mut cow_vars: Vec<VarId> = Vec::new(); // Track CoW vars for cleanup

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

    fn generate_instruction(
        &self,
        instr: &MirInstr,
        variables: &mut HashMap<VarId, VarStorage<'ctx>>,
        _cow_vars: &mut Vec<VarId>,
    ) -> CodegenResult<()> {
        match instr {
            MirInstr::Assign { dest, source } => {
                let value = self.operand_to_value(source, variables)?;

                // For MVP, treat as Copy type (SSA)
                // TODO: check actual type, if CoW then use alloca + clone
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
                // Handle builtin functions with special behavior
                if func == "print" {
                    // print() supports multiple arguments - print each one, then newline
                    for (i, arg) in args.iter().enumerate() {
                        let arg_value = self.operand_to_value(arg, variables)?;

                        // Add space between arguments (except first)
                        if i > 0 {
                            let space_fn =
                                self.module.get_function("kiv_print_text").ok_or_else(|| {
                                    CodegenError::UndefinedFunction("kiv_print_text".to_string())
                                })?;
                            let space_str = self
                                .builder
                                .build_global_string_ptr(" ", "space")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                            let space_text_fn = self
                                .module
                                .get_function("kiv_text_from_cstr")
                                .ok_or_else(|| {
                                    CodegenError::UndefinedFunction(
                                        "kiv_text_from_cstr".to_string(),
                                    )
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

                        let print_fn =
                            self.module.get_function(print_fn_name).ok_or_else(|| {
                                CodegenError::UndefinedFunction(print_fn_name.to_string())
                            })?;

                        self.builder
                            .build_call(print_fn, &[arg_value.into()], "")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }

                    // Print newline at the end
                    let newline_fn =
                        self.module
                            .get_function("kiv_print_newline")
                            .ok_or_else(|| {
                                CodegenError::UndefinedFunction("kiv_print_newline".to_string())
                            })?;
                    self.builder
                        .build_call(newline_fn, &[], "")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                } else {
                    // Handle other function calls
                    let runtime_func_name = match func.as_str() {
                        "panic" => "kiv_panic",
                        other => other,
                    };

                    let function =
                        self.module.get_function(runtime_func_name).ok_or_else(|| {
                            CodegenError::UndefinedFunction(format!(
                                "{} (mapped from {})",
                                runtime_func_name, func
                            ))
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
                }
            }

            MirInstr::Nop => {}
        }

        Ok(())
    }

    fn drop_cow_variables(&self, cow_vars: &[VarId]) -> CodegenResult<()> {
        // For each CoW variable, insert a drop call
        for _var_id in cow_vars {
            // TODO: implement when we have actual CoW types
            // let drop_fn = self.module.get_function("kiv_text_drop").unwrap();
            // self.builder.build_call(drop_fn, &[var_ptr.into()], "");
        }
        Ok(())
    }

    fn generate_terminator(
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

    fn operand_to_value(
        &self,
        operand: &MirOperand,
        variables: &HashMap<VarId, VarStorage<'ctx>>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        match operand {
            MirOperand::Var(var_id) => {
                match variables.get(var_id) {
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
                }
            }
            MirOperand::Literal(lit) => self.literal_to_value(lit),
            MirOperand::Unit => Ok(self.context.i64_type().const_zero().into()),
        }
    }

    fn literal_to_value(&self, lit: &Literal) -> CodegenResult<BasicValueEnum<'ctx>> {
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

    fn build_binop(
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

    #[allow(dead_code)]
    fn is_copy_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Int | Type::Float | Type::Bool | Type::Unit)
    }
}
