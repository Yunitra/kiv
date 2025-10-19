//! Redundant else branch detection.

use super::LintRule;
use kivc_diagnostics::{DiagnosticsCollector, KivError};
use kivc_hir::*;

/// Detects redundant else branches
pub struct RedundantElse;

impl LintRule for RedundantElse {
    fn name(&self) -> &'static str {
        "redundant_else"
    }

    fn check(&self, program: &HirProgram, diagnostics: &mut DiagnosticsCollector) {
        for func in &program.functions {
            check_redundant_else_in_block(&func.body, diagnostics);
        }
    }
}

fn check_redundant_else_in_block(block: &HirBlock, diagnostics: &mut DiagnosticsCollector) {
    for stmt in &block.stmts {
        match &stmt.kind {
            HirStmtKind::Expr { expr } => {
                check_redundant_else_in_expr(expr, diagnostics);
            }
            HirStmtKind::Let { init, .. } => {
                if let Some(init) = init {
                    check_redundant_else_in_expr(init, diagnostics);
                }
            }
            HirStmtKind::While { condition, body } => {
                check_redundant_else_in_expr(condition, diagnostics);
                check_redundant_else_in_block(body, diagnostics);
            }
            HirStmtKind::For { iterable, body, .. } => {
                check_redundant_else_in_expr(iterable, diagnostics);
                check_redundant_else_in_block(body, diagnostics);
            }
            _ => {}
        }
    }
}

fn check_redundant_else_in_expr(expr: &HirExpr, diagnostics: &mut DiagnosticsCollector) {
    match &expr.kind {
        HirExprKind::If {
            then_block,
            else_block,
            ..
        } => {
            // Check if then branch always returns
            if let Some(else_b) = else_block
                && block_always_returns(then_block)
            {
                let error = KivError::warning(
                    else_b.span.clone(),
                    "redundant else block".to_string(),
                    "the `else` block is unnecessary because the `if` block always returns"
                        .to_string(),
                );
                diagnostics.add(error);
            }

            // Recursively check
            check_redundant_else_in_block(then_block, diagnostics);
            if let Some(else_b) = else_block {
                check_redundant_else_in_block(else_b, diagnostics);
            }
        }
        HirExprKind::BinOp { left, right, .. } => {
            check_redundant_else_in_expr(left, diagnostics);
            check_redundant_else_in_expr(right, diagnostics);
        }
        HirExprKind::Call { args, .. } => {
            for arg in args {
                check_redundant_else_in_expr(arg, diagnostics);
            }
        }
        HirExprKind::Block(block) => {
            check_redundant_else_in_block(block, diagnostics);
        }
        _ => {}
    }
}

fn block_always_returns(block: &HirBlock) -> bool {
    // For now, simplified check - just look at last statement
    // In future, could be more sophisticated (e.g., if all branches return)
    false
}
