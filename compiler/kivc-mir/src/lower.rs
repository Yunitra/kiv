//! Lowering from HIR to MIR.

use crate::block::{BasicBlock, BlockBuilder, BlockId};
use crate::mir::{MirFunction, MirInstr, MirOperand, MirProgram, MirTerminator};
use kivc_hir::{HirBlock, HirExpr, HirExprKind, HirFunDef, HirProgram, HirStmtKind, TypeId, VarId};
use kivc_typeck::Type;
use std::collections::HashMap;

/// Lowers a HIR program to MIR
pub fn lower_to_mir(program: HirProgram) -> MirProgram {
    let mut lowerer = MirLowerer::new();
    lowerer.lower_program(program)
}

/// The lowering context
struct MirLowerer {
    block_builder: BlockBuilder,
    type_table: HashMap<TypeId, Type>,
}

impl MirLowerer {
    fn new() -> Self {
        Self {
            block_builder: BlockBuilder::new(),
            type_table: HashMap::new(),
        }
    }

    fn lower_program(&mut self, program: HirProgram) -> MirProgram {
        // Filter out built-in functions (print, panic) - they're handled specially in codegen
        let user_functions: Vec<_> = program
            .functions
            .into_iter()
            .filter(|f| f.name != "print" && f.name != "panic")
            .collect();

        // Build type table from all functions' parameters and return types
        for func in &user_functions {
            for param in &func.params {
                self.register_type(param.ty);
            }
            if let Some(ret_ty) = func.return_type {
                self.register_type(ret_ty);
            }
        }

        let functions = user_functions
            .into_iter()
            .map(|f| self.lower_function(f))
            .collect();

        MirProgram::new(functions, self.type_table.clone())
    }

    /// Registers a TypeId in the type table
    fn register_type(&mut self, type_id: TypeId) {
        // TypeId mapping from TypeContext (see kivc-typeck/src/types.rs)
        // TypeId(0) = Int, TypeId(1) = Float, TypeId(2) = Bool, TypeId(3) = Text, TypeId(4) = Unit
        let ty = match type_id.as_usize() {
            0 => Type::Int,
            1 => Type::Float,
            2 => Type::Bool,
            3 => Type::Text,
            4 => Type::Unit,
            _ => Type::Unknown,
        };
        self.type_table.insert(type_id, ty);
    }

    fn lower_function(&mut self, fun: HirFunDef) -> MirFunction {
        let entry_block_id = self.block_builder.alloc_block_id();

        let params = fun.params.iter().map(|p| p.var_id).collect();
        let param_types = fun.params.iter().map(|p| p.ty).collect();

        let mut mir_fun = MirFunction::new(
            fun.id,
            fun.name,
            params,
            param_types,
            fun.return_type,
            entry_block_id,
        );

        // Lower the function body
        let (entry_block, _) = self.lower_block(fun.body, entry_block_id, None);
        mir_fun.add_block(entry_block);

        mir_fun
    }

    fn lower_block(
        &mut self,
        block: HirBlock,
        block_id: BlockId,
        next_block: Option<BlockId>,
    ) -> (BasicBlock, Option<VarId>) {
        let mut current_block = BasicBlock::new(block_id, MirTerminator::Unreachable);
        let mut last_value: Option<VarId> = None;

        for stmt in block.stmts {
            match stmt.kind {
                HirStmtKind::Let { var_id, init, .. } => {
                    let operand = self.lower_expr_to_operand(&init, &mut current_block);
                    current_block.push_instr(MirInstr::Assign {
                        dest: var_id,
                        source: operand,
                    });
                    last_value = Some(var_id);
                }

                HirStmtKind::Const { var_id, value, .. } => {
                    let operand = self.lower_expr_to_operand(&value, &mut current_block);
                    current_block.push_instr(MirInstr::Assign {
                        dest: var_id,
                        source: operand,
                    });
                    last_value = Some(var_id);
                }

                HirStmtKind::Return { value } => {
                    let ret_operand = value
                        .as_ref()
                        .map(|v| self.lower_expr_to_operand(v, &mut current_block));

                    current_block.terminator = MirTerminator::Return { value: ret_operand };
                    return (current_block, last_value);
                }

                HirStmtKind::While { .. } => {
                    // requires loop context management and is deferred to future iterations
                    current_block.push_instr(MirInstr::Nop);
                }

                HirStmtKind::For { .. } => {
                    current_block.push_instr(MirInstr::Nop);
                }

                HirStmtKind::Break => {
                    // Break: terminates execution (simplified)
                    // NOTE: Proper implementation requires tracking loop exit block during lowering
                }

                HirStmtKind::Continue => {
                    // Continue: terminates execution (simplified)
                    // NOTE: Proper implementation requires tracking loop header block during lowering
                    current_block.terminator = MirTerminator::Unreachable;

                HirStmtKind::Expr { expr } => {
                    // Evaluate expression for side effects
                    let _ = self.lower_expr_to_operand(&expr, &mut current_block);
                }
            }
        }

        // If no explicit return, jump to next block or return void
        current_block.terminator = match next_block {
            Some(next) => MirTerminator::Jump { target: next },
            None => MirTerminator::Return { value: None },
        };

        (current_block, last_value)
    }

    fn lower_expr_to_operand(&mut self, expr: &HirExpr, block: &mut BasicBlock) -> MirOperand {
        match &expr.kind {
            HirExprKind::Literal(lit) => MirOperand::Literal(lit.clone()),

            HirExprKind::Var { var_id, .. } => MirOperand::Var(*var_id),

            HirExprKind::Binary { op, lhs, rhs } => {
                let left_op = self.lower_expr_to_operand(lhs, block);
                let right_op = self.lower_expr_to_operand(rhs, block);

                // Allocate a temporary variable for the result
                let dest = VarId::new(self.next_temp_id());

                block.push_instr(MirInstr::BinOp {
                    dest,
                    op: *op,
                    left: left_op,
                    right: right_op,
                });

                MirOperand::Var(dest)
            }

            HirExprKind::Call { func, args } => {
                let arg_operands: Vec<_> = args
                    .iter()
                    .map(|arg| self.lower_expr_to_operand(arg, block))
                    .collect();

                // Allocate a temporary for the result
                let dest = VarId::new(self.next_temp_id());

                block.push_instr(MirInstr::Call {
                    dest: Some(dest),
                    func: func.clone(),
                    args: arg_operands,
                });

                MirOperand::Var(dest)
            }

            HirExprKind::Assign { var_id, value, .. } => {
                let value_op = self.lower_expr_to_operand(value, block);

                block.push_instr(MirInstr::Assign {
                    dest: *var_id,
                    source: value_op,
                });

                MirOperand::Unit
            }

            HirExprKind::Match { .. } => {
                // Match: simplified implementation returns Unit
                // NOTE: Full implementation requires pattern matching decision tree,
                // binding extraction, guard evaluation, and multi-block CFG
                MirOperand::Unit
            }

                for stmt in &inner_block.stmts {
                    match &stmt.kind {
                        HirStmtKind::Let { var_id, init, .. } => {
                            let operand = self.lower_expr_to_operand(init, block);
                            block.push_instr(MirInstr::Assign {
                                dest: *var_id,
                                source: operand,
                            });
                        }
                        HirStmtKind::Const { var_id, value, .. } => {
                            let operand = self.lower_expr_to_operand(value, block);
                            block.push_instr(MirInstr::Assign {
                                dest: *var_id,
                                source: operand,
                            });
                        }
                        HirStmtKind::Expr { expr } => {
                            let _ = self.lower_expr_to_operand(expr, block);
                        }
                        HirStmtKind::Return { .. }
                        | HirStmtKind::While { .. }
                        | HirStmtKind::For { .. }
                        | HirStmtKind::Break
                        | HirStmtKind::Continue => {
                            // These require special handling
                        }
                    }
                }
                MirOperand::Unit
            }

            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                // For MVP, we simplify if-expressions by linearizing them
                let cond_op = self.lower_expr_to_operand(cond, block);

                let then_block_id = self.block_builder.alloc_block_id();
                let else_block_id = self.block_builder.alloc_block_id();
                let merge_block_id = self.block_builder.alloc_block_id();

                // Set current block terminator to branch
                block.terminator = MirTerminator::Branch {
                    condition: cond_op,
                    true_block: then_block_id,
                    false_block: else_block_id,
                };

                // Lower then branch
                let (_then_block, _) =
                    self.lower_block(then_branch.clone(), then_block_id, Some(merge_block_id));

                // Lower else branch if it exists
                if let Some(else_b) = else_branch {
                    let (_else_block, _) =
                        self.lower_block(else_b.clone(), else_block_id, Some(merge_block_id));
                } else {
                    // Empty else: just jump to merge
                    let _else_block = BasicBlock::new(
                        else_block_id,
                        MirTerminator::Jump {
                            target: merge_block_id,
                        },
                    );
                }

                // For MVP, we return Unit for if-expressions
                MirOperand::Unit
            }
        }
    }

    fn next_temp_id(&mut self) -> usize {
        // Simple temporary ID allocation
        // In a real compiler, this would be more sophisticated
        self.block_builder.alloc_block_id().as_usize() + 10000
    }
}
