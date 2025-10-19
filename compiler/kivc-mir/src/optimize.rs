//! MIR optimization passes.
//!
//! This module implements various optimization passes for MIR:
//! - Constant folding
//! - Dead code elimination (DCE)
//! - Copy propagation
//! - Instruction combining

use crate::mir::{MirFunction, MirInstr, MirOperand, MirProgram};
use kivc_ast::{BinOp, Literal};
use std::collections::{HashMap, HashSet};

/// Runs all optimization passes on a MIR program
pub fn optimize_program(mut program: MirProgram, opt_level: OptLevel) -> MirProgram {
    if opt_level == OptLevel::None {
        return program;
    }

    for function in &mut program.functions {
        optimize_function(function, opt_level);
    }

    program
}

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    None,
    Basic,
    Aggressive,
}

fn optimize_function(function: &mut MirFunction, opt_level: OptLevel) {
    // Apply optimization passes in order
    constant_fold_function(function);

    if opt_level == OptLevel::Aggressive {
        dead_code_elimination(function);
        copy_propagation(function);
    }
}

/// Constant folding: evaluate constant expressions at compile time
fn constant_fold_function(function: &mut MirFunction) {
    for block in &mut function.blocks {
        for instr in &mut block.instructions {
            if let MirInstr::BinOp {
                dest,
                op,
                left,
                right,
            } = instr
                && let (MirOperand::Literal(l_lit), MirOperand::Literal(r_lit)) = (left, right)
                && let Some(result) = fold_binary_op(*op, l_lit, r_lit)
            {
                *instr = MirInstr::Assign {
                    dest: *dest,
                    source: MirOperand::Literal(result),
                };
            }
        }
    }
}

fn fold_binary_op(op: BinOp, left: &Literal, right: &Literal) -> Option<Literal> {
    match (left, right) {
        (Literal::Int(l), Literal::Int(r)) => {
            let result = match op {
                BinOp::Add => l.checked_add(*r)?,
                BinOp::Sub => l.checked_sub(*r)?,
                BinOp::Mul => l.checked_mul(*r)?,
                BinOp::Div => l.checked_div(*r)?,
                BinOp::Eq => return Some(Literal::Bool(l == r)),
                BinOp::NotEq => return Some(Literal::Bool(l != r)),
                BinOp::Lt => return Some(Literal::Bool(l < r)),
                BinOp::Le => return Some(Literal::Bool(l <= r)),
                BinOp::Gt => return Some(Literal::Bool(l > r)),
                BinOp::Ge => return Some(Literal::Bool(l >= r)),
            };
            Some(Literal::Int(result))
        }
        (Literal::Float(l), Literal::Float(r)) => {
            let result = match op {
                BinOp::Add => l + r,
                BinOp::Sub => l - r,
                BinOp::Mul => l * r,
                BinOp::Div => l / r,
                BinOp::Eq => return Some(Literal::Bool(l == r)),
                BinOp::NotEq => return Some(Literal::Bool(l != r)),
                BinOp::Lt => return Some(Literal::Bool(l < r)),
                BinOp::Le => return Some(Literal::Bool(l <= r)),
                BinOp::Gt => return Some(Literal::Bool(l > r)),
                BinOp::Ge => return Some(Literal::Bool(l >= r)),
            };
            Some(Literal::Float(result))
        }
        (Literal::Bool(l), Literal::Bool(r)) => match op {
            BinOp::Eq => Some(Literal::Bool(l == r)),
            BinOp::NotEq => Some(Literal::Bool(l != r)),
            _ => None,
        },
        _ => None,
    }
}

/// Dead code elimination: remove instructions that have no effect
fn dead_code_elimination(function: &mut MirFunction) {
    let mut used_vars = HashSet::new();

    // Mark variables used in terminators
    for block in &function.blocks {
        match &block.terminator {
            crate::mir::MirTerminator::Return { value: Some(op) } => {
                mark_operand_used(op, &mut used_vars);
            }
            crate::mir::MirTerminator::Branch { condition, .. } => {
                mark_operand_used(condition, &mut used_vars);
            }
            _ => {}
        }
    }

    // Backward pass: mark variables that contribute to used variables
    let mut changed = true;
    while changed {
        changed = false;
        for block in &function.blocks {
            for instr in block.instructions.iter().rev() {
                match instr {
                    MirInstr::Assign { dest, source } => {
                        if used_vars.contains(dest) {
                            changed |= mark_operand_used(source, &mut used_vars);
                        }
                    }
                    MirInstr::BinOp {
                        dest, left, right, ..
                    } => {
                        if used_vars.contains(dest) {
                            changed |= mark_operand_used(left, &mut used_vars);
                            changed |= mark_operand_used(right, &mut used_vars);
                        }
                    }
                    MirInstr::Call {
                        dest: Some(dest),
                        args,
                        ..
                    } => {
                        if used_vars.contains(dest) {
                            for arg in args {
                                changed |= mark_operand_used(arg, &mut used_vars);
                            }
                        }
                    }
                    MirInstr::Call {
                        dest: None, args, ..
                    } => {
                        // Function calls with no return value (side effects) - keep args
                        for arg in args {
                            changed |= mark_operand_used(arg, &mut used_vars);
                        }
                    }
                    MirInstr::Nop => {}
                }
            }
        }
    }

    // Remove dead instructions
    for block in &mut function.blocks {
        block.instructions.retain(|instr| {
            match instr {
                MirInstr::Assign { dest, .. } | MirInstr::BinOp { dest, .. } => {
                    used_vars.contains(dest)
                }
                MirInstr::Call {
                    dest: Some(dest), ..
                } => used_vars.contains(dest),
                MirInstr::Call { dest: None, .. } => true, // Keep side-effecting calls
                MirInstr::Nop => false,
            }
        });
    }
}

fn mark_operand_used(operand: &MirOperand, used: &mut HashSet<crate::VarId>) -> bool {
    match operand {
        MirOperand::Var(var_id) => used.insert(*var_id),
        _ => false,
    }
}

/// Copy propagation: replace variable uses with their constant values
fn copy_propagation(function: &mut MirFunction) {
    let mut copy_map: HashMap<crate::VarId, MirOperand> = HashMap::new();

    for block in &mut function.blocks {
        for instr in &mut block.instructions {
            // First, replace operands using the copy map
            match instr {
                MirInstr::Assign { source, .. } => {
                    *source = propagate_operand(source, &copy_map);
                }
                MirInstr::BinOp { left, right, .. } => {
                    *left = propagate_operand(left, &copy_map);
                    *right = propagate_operand(right, &copy_map);
                }
                MirInstr::Call { args, .. } => {
                    for arg in args {
                        *arg = propagate_operand(arg, &copy_map);
                    }
                }
                MirInstr::Nop => {}
            }

            // Then, update the copy map based on this instruction
            if let MirInstr::Assign { dest, source } = instr {
                match source {
                    MirOperand::Literal(_) | MirOperand::Unit => {
                        copy_map.insert(*dest, source.clone());
                    }
                    MirOperand::Var(src_var) => {
                        // Transitively propagate if source is also a copy
                        if let Some(transitive) = copy_map.get(src_var) {
                            copy_map.insert(*dest, transitive.clone());
                        }
                    }
                }
            }
        }
    }
}

fn propagate_operand(
    operand: &MirOperand,
    copy_map: &HashMap<crate::VarId, MirOperand>,
) -> MirOperand {
    match operand {
        MirOperand::Var(var_id) => copy_map
            .get(var_id)
            .cloned()
            .unwrap_or_else(|| operand.clone()),
        _ => operand.clone(),
    }
}
