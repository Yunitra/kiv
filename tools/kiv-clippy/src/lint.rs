//! Lint rules for Kiv code.

use kivc_diagnostics::{DiagnosticsCollector, KivError};
use kivc_hir::*;
use std::collections::{HashMap, HashSet};

/// A lint rule
pub trait LintRule {
    /// The name of the lint rule
    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    /// Runs the lint rule on a HIR program
    fn check(&self, program: &HirProgram, diagnostics: &mut DiagnosticsCollector);
}

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

            // Collect declared variables from the function body
            collect_declarations(&func.body, &mut declared);

            // Collect used variables
            collect_usages(&func.body, &mut used);

            // Report unused variables
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

/// Detects non-literal const values
pub struct ConstMutable;

impl LintRule for ConstMutable {
    fn name(&self) -> &'static str {
        "const_mutable"
    }

    fn check(&self, program: &HirProgram, _diagnostics: &mut DiagnosticsCollector) {
        // This would require tracking const declarations in HIR
        // For now, this is a placeholder
        for _func in &program.functions {
            // Check const declarations
        }
    }
}

// Helper functions

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
            then_branch,
            else_branch,
            ..
        } => {
            collect_declarations(then_branch, declared);
            if let Some(else_branch) = else_branch {
                collect_declarations(else_branch, declared);
            }
        }
        HirExprKind::Binary { lhs, rhs, .. } => {
            collect_declarations_from_expr(lhs, declared);
            collect_declarations_from_expr(rhs, declared);
        }
        _ => {}
    }
}

fn collect_usages(block: &HirBlock, used: &mut HashSet<VarId>) {
    for stmt in &block.stmts {
        match &stmt.kind {
            HirStmtKind::Let { init, .. } => {
                collect_usages_from_expr(init, used);
            }
            HirStmtKind::Const { value, .. } => {
                collect_usages_from_expr(value, used);
            }
            HirStmtKind::Return { value } => {
                if let Some(value) = value {
                    collect_usages_from_expr(value, used);
                }
            }
            HirStmtKind::While { condition, body } => {
                collect_usages_from_expr(condition, used);
                collect_usages(body, used);
            }
            HirStmtKind::For { iterable, body, .. } => {
                collect_usages_from_expr(iterable, used);
                collect_usages(body, used);
            }
            HirStmtKind::Break | HirStmtKind::Continue => {}
            HirStmtKind::Expr { expr } => {
                collect_usages_from_expr(expr, used);
            }
        }
    }
}

fn collect_usages_from_expr(expr: &HirExpr, used: &mut HashSet<VarId>) {
    match &expr.kind {
        HirExprKind::Var { var_id, .. } => {
            used.insert(*var_id);
        }
        HirExprKind::Binary { lhs, rhs, .. } => {
            collect_usages_from_expr(lhs, used);
            collect_usages_from_expr(rhs, used);
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_usages_from_expr(cond, used);
            collect_usages(then_branch, used);
            if let Some(else_branch) = else_branch {
                collect_usages(else_branch, used);
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
        HirExprKind::Call { args, .. } => {
            for arg in args {
                collect_usages_from_expr(arg, used);
            }
        }
        HirExprKind::Assign { value, .. } => {
            collect_usages_from_expr(value, used);
        }
        _ => {}
    }
}

fn check_unreachable_in_block(block: &HirBlock, diagnostics: &mut DiagnosticsCollector) {
    let mut after_return = false;

    for stmt in &block.stmts {
        if after_return {
            let error = KivError::warning(
                stmt.span.clone(),
                "unreachable code".to_string(),
                "any code after a `return` statement will never be executed".to_string(),
            );
            diagnostics.add(error);
        }

        if matches!(stmt.kind, HirStmtKind::Return { .. }) {
            after_return = true;
        }

        // Recursively check expressions
        match &stmt.kind {
            HirStmtKind::Expr { expr } => {
                check_unreachable_in_expr(expr, diagnostics);
            }
            HirStmtKind::Let { init, .. } => {
                check_unreachable_in_expr(init, diagnostics);
            }
            HirStmtKind::Const { value, .. } => {
                check_unreachable_in_expr(value, diagnostics);
            }
            HirStmtKind::Return { value } => {
                if let Some(value) = value {
                    check_unreachable_in_expr(value, diagnostics);
                }
            }
            HirStmtKind::While { condition, body } => {
                check_unreachable_in_expr(condition, diagnostics);
                check_unreachable_in_block(body, diagnostics);
            }
            HirStmtKind::For { iterable, body, .. } => {
                check_unreachable_in_expr(iterable, diagnostics);
                check_unreachable_in_block(body, diagnostics);
            }
            HirStmtKind::Break | HirStmtKind::Continue => {}
        }
    }
}

fn check_unreachable_in_expr(expr: &HirExpr, diagnostics: &mut DiagnosticsCollector) {
    match &expr.kind {
        HirExprKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            check_unreachable_in_block(then_branch, diagnostics);
            if let Some(else_branch) = else_branch {
                check_unreachable_in_block(else_branch, diagnostics);
            }
        }
        HirExprKind::Match { arms, .. } => {
            for arm in arms {
                check_unreachable_in_expr(&arm.body, diagnostics);
            }
        }
        HirExprKind::Block(block) => {
            check_unreachable_in_block(block, diagnostics);
        }
        HirExprKind::Binary { lhs, rhs, .. } => {
            check_unreachable_in_expr(lhs, diagnostics);
            check_unreachable_in_expr(rhs, diagnostics);
        }
        HirExprKind::Call { args, .. } => {
            for arg in args {
                check_unreachable_in_expr(arg, diagnostics);
            }
        }
        _ => {}
    }
}

fn check_redundant_else_in_block(block: &HirBlock, diagnostics: &mut DiagnosticsCollector) {
    for stmt in &block.stmts {
        match &stmt.kind {
            HirStmtKind::Expr { expr } => {
                check_redundant_else_in_expr(expr, diagnostics);
            }
            HirStmtKind::Let { init, .. } => {
                check_redundant_else_in_expr(init, diagnostics);
            }
            HirStmtKind::Const { value, .. } => {
                check_redundant_else_in_expr(value, diagnostics);
            }
            HirStmtKind::While { condition, body } => {
                check_redundant_else_in_expr(condition, diagnostics);
                check_redundant_else_in_block(body, diagnostics);
            }
            HirStmtKind::For { iterable, body, .. } => {
                check_redundant_else_in_expr(iterable, diagnostics);
                check_redundant_else_in_block(body, diagnostics);
            }
            HirStmtKind::Break | HirStmtKind::Continue => {}
            HirStmtKind::Return { value } => {
                if let Some(value) = value {
                    check_redundant_else_in_expr(value, diagnostics);
                }
            }
        }
    }
}

fn check_redundant_else_in_expr(expr: &HirExpr, diagnostics: &mut DiagnosticsCollector) {
    match &expr.kind {
        HirExprKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            // Check if then branch always returns
            if let Some(else_branch) = else_branch
                && block_always_returns(then_branch)
            {
                let error = KivError::warning(
                    else_branch.span.clone(),
                    "redundant else block".to_string(),
                    "the `else` block is unnecessary because the `if` block always returns"
                        .to_string(),
                );
                diagnostics.add(error);
            }

            // Recursively check
            check_redundant_else_in_block(then_branch, diagnostics);
            if let Some(else_branch) = else_branch {
                check_redundant_else_in_block(else_branch, diagnostics);
            }
        }
        HirExprKind::Binary { lhs, rhs, .. } => {
            check_redundant_else_in_expr(lhs, diagnostics);
            check_redundant_else_in_expr(rhs, diagnostics);
        }
        HirExprKind::Call { args, .. } => {
            for arg in args {
                check_redundant_else_in_expr(arg, diagnostics);
            }
        }
        _ => {}
    }
}

fn block_always_returns(block: &HirBlock) -> bool {
    block
        .stmts
        .last()
        .map(|stmt| matches!(stmt.kind, HirStmtKind::Return { .. }))
        .unwrap_or(false)
}

/// Returns all available lint rules
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(UnusedVariable),
        Box::new(UnreachableCode),
        Box::new(RedundantElse),
        Box::new(ConstMutable),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_have_unique_names() {
        let rules = all_rules();
        let mut names = HashSet::new();

        for rule in rules {
            assert!(
                names.insert(rule.name()),
                "Duplicate rule name: {}",
                rule.name()
            );
        }
    }
}
