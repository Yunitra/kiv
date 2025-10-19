//! Unused variable detection.

use super::LintRule;
use kivc_diagnostics::{DiagnosticsCollector, KivError};
use kivc_hir::*;
use std::collections::{HashMap, HashSet};

/// Detects unused variables
pub struct UnusedVariable;

impl LintRule for UnusedVariable {
    fn name(&self) -> &'static str {
        "unused_variable"
    }

    fn check(&self, program: &HirProgram, diagnostics: &mut DiagnosticsCollector) {
        for func in &program.functions {
            let mut declared = HashMap::new();
            let mut used = HashSet::new();

            collect_declarations(&func.body, &mut declared);
            collect_usages(&func.body, &mut used);

            for (var_id, (name, span)) in declared {
                if !used.contains(&var_id) && !name.starts_with('_') {
                    let error = KivError::warning(
                        span.clone(),
                        format!("unused variable: `{}`", name),
                        "consider prefixing with `_` if intentional".to_string(),
                    );
                    diagnostics.add(error);
                }
            }
        }
    }
}

fn collect_declarations(
    block: &HirBlock,
    declared: &mut HashMap<VarId, (String, kivc_span::Span)>,
) {
    for stmt in &block.stmts {
        match &stmt.kind {
            HirStmtKind::Let { var_id, name, .. } => {
                declared.insert(*var_id, (name.clone(), stmt.span.clone()));
            }
            HirStmtKind::Expr { expr } => {
                collect_declarations_from_expr(expr, declared);
            }
            HirStmtKind::While { body, .. } | HirStmtKind::For { body, .. } => {
                collect_declarations(body, declared);
            }
            _ => {}
        }
    }
}

fn collect_declarations_from_expr(
    expr: &HirExpr,
    declared: &mut HashMap<VarId, (String, kivc_span::Span)>,
) {
    match &expr.kind {
        HirExprKind::If {
            then_block,
            else_block,
            ..
        } => {
            collect_declarations(then_block, declared);
            if let Some(else_block) = else_block {
                collect_declarations(else_block, declared);
            }
        }
        HirExprKind::Block(block) => {
            collect_declarations(block, declared);
        }
        HirExprKind::Match { arms, .. } => {
            for arm in arms {
                if let HirPattern::Binding { var_id, name } = &arm.pattern {
                    declared.insert(*var_id, (name.clone(), arm.span.clone()));
                }
            }
        }
        _ => {}
    }
}

fn collect_usages(block: &HirBlock, used: &mut HashSet<VarId>) {
    for stmt in &block.stmts {
        match &stmt.kind {
            HirStmtKind::Let { init, .. } => {
                if let Some(init) = init {
                    collect_usages_from_expr(init, used);
                }
            }
            HirStmtKind::Expr { expr } => {
                collect_usages_from_expr(expr, used);
            }
            HirStmtKind::While { condition, body } => {
                collect_usages_from_expr(condition, used);
                collect_usages(body, used);
            }
            HirStmtKind::For { iterable, body, .. } => {
                collect_usages_from_expr(iterable, used);
                collect_usages(body, used);
            }
            HirStmtKind::Assign { target, value } => {
                used.insert(*target);
                collect_usages_from_expr(value, used);
            }
            HirStmtKind::Break | HirStmtKind::Continue => {}
        }
    }
}

fn collect_usages_from_expr(expr: &HirExpr, used: &mut HashSet<VarId>) {
    match &expr.kind {
        HirExprKind::Var { var_id, .. } => {
            used.insert(*var_id);
        }
        HirExprKind::BinOp { left, right, .. } => {
            collect_usages_from_expr(left, used);
            collect_usages_from_expr(right, used);
        }
        HirExprKind::Call { args, .. } => {
            for arg in args {
                collect_usages_from_expr(arg, used);
            }
        }
        HirExprKind::If {
            condition,
            then_block,
            else_block,
        } => {
            collect_usages_from_expr(condition, used);
            collect_usages(then_block, used);
            if let Some(else_block) = else_block {
                collect_usages(else_block, used);
            }
        }
        HirExprKind::Match { value, arms } => {
            collect_usages_from_expr(value, used);
            for arm in arms {
                collect_usages_from_expr(&arm.body, used);
            }
        }
        HirExprKind::Block(block) => {
            collect_usages(block, used);
        }
        HirExprKind::Literal(_) => {}
    }
}
