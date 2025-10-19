//! Unreachable code detection.

use super::LintRule;
use kivc_diagnostics::{DiagnosticsCollector, KivError};
use kivc_hir::*;

/// Detects unreachable code
pub struct UnreachableCode;

impl LintRule for UnreachableCode {
    fn name(&self) -> &'static str {
        "unreachable_code"
    }

    fn check(&self, program: &HirProgram, diagnostics: &mut DiagnosticsCollector) {
        for func in &program.functions {
            check_unreachable_in_block(&func.body, diagnostics);
        }
    }
}

fn check_unreachable_in_block(block: &HirBlock, diagnostics: &mut DiagnosticsCollector) {
    let mut found_return = false;

    for (i, stmt) in block.stmts.iter().enumerate() {
        if found_return {
            let error = KivError::warning(
                stmt.span.clone(),
                "unreachable statement".to_string(),
                "this statement will never be executed".to_string(),
            );
            diagnostics.add(error);
            break;
        }

        match &stmt.kind {
            HirStmtKind::Expr { expr } => {
                if is_diverging_expr(expr) {
                    found_return = true;
                    if i + 1 < block.stmts.len() {
                        continue;
                    }
                }
                check_unreachable_in_expr(expr, diagnostics);
            }
            HirStmtKind::While { condition, body } => {
                check_unreachable_in_expr(condition, diagnostics);
                check_unreachable_in_block(body, diagnostics);
            }
            HirStmtKind::For { iterable, body, .. } => {
                check_unreachable_in_expr(iterable, diagnostics);
                check_unreachable_in_block(body, diagnostics);
            }
            _ => {}
        }
    }
}

fn check_unreachable_in_expr(expr: &HirExpr, diagnostics: &mut DiagnosticsCollector) {
    match &expr.kind {
        HirExprKind::If {
            condition,
            then_block,
            else_block,
        } => {
            check_unreachable_in_expr(condition, diagnostics);
            check_unreachable_in_block(then_block, diagnostics);
            if let Some(else_block) = else_block {
                check_unreachable_in_block(else_block, diagnostics);
            }
        }
        HirExprKind::Match { value, arms } => {
            check_unreachable_in_expr(value, diagnostics);
            for arm in arms {
                check_unreachable_in_expr(&arm.body, diagnostics);
            }
        }
        HirExprKind::Block(block) => {
            check_unreachable_in_block(block, diagnostics);
        }
        HirExprKind::BinOp { left, right, .. } => {
            check_unreachable_in_expr(left, diagnostics);
            check_unreachable_in_expr(right, diagnostics);
        }
        _ => {}
    }
}

fn is_diverging_expr(expr: &HirExpr) -> bool {
    match &expr.kind {
        HirExprKind::Call { name, .. } => name == "panic" || name == "exit",
        _ => false,
    }
}
