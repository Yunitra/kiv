//! Statement type checking.

use super::context::TypeChecker;
use crate::types::Type;
use kivc_diagnostics::KivError;
use kivc_hir::{HirBlock, HirStmt, HirStmtKind};

impl TypeChecker {
    pub(super) fn check_block(&mut self, mut block: HirBlock) -> HirBlock {
        block.stmts = block
            .stmts
            .into_iter()
            .map(|s| self.check_stmt(s))
            .collect();

        block
    }

    pub(super) fn check_stmt(&mut self, mut stmt: HirStmt) -> HirStmt {
        match &mut stmt.kind {
            HirStmtKind::Let {
                var_id,
                mutable,
                ty,
                init,
                ..
            } => {
                *init = self.check_expr(init.clone());

                // Infer or check type
                let init_type = self.infer_expr_type(init);

                let var_type = if let Some(ty_id) = ty {
                    // Explicit type annotation
                    let expected_type = self.ctx.get(*ty_id).cloned().unwrap_or(Type::Unknown);

                    if !self.ctx.are_compatible(&expected_type, &init_type) {
                        self.diagnostics.add(KivError::type_error(
                            &init.span,
                            format!(
                                "type mismatch: expected {}, found {}",
                                expected_type.name(),
                                init_type.name()
                            ),
                            "incompatible type",
                            None,
                            kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                        ));
                    }

                    expected_type
                } else {
                    // Type inference
                    init_type
                };

                self.var_types.insert(*var_id, var_type);
                self.var_mutability.insert(*var_id, *mutable);
            }

            HirStmtKind::Const { var_id, value, .. } => {
                *value = self.check_expr(value.clone());

                let value_type = self.infer_expr_type(value);
                self.var_types.insert(*var_id, value_type);
                self.var_mutability.insert(*var_id, false);
            }

            HirStmtKind::Return { value } => {
                if let Some(ret_expr) = value {
                    *ret_expr = self.check_expr(ret_expr.clone());

                    let ret_type = self.infer_expr_type(ret_expr);

                    // Check return type matches function signature
                    if let Some(expected_ty_id) = self.current_return_type {
                        let expected_type = self
                            .ctx
                            .get(expected_ty_id)
                            .cloned()
                            .unwrap_or(Type::Unknown);

                        if !self.ctx.are_compatible(&expected_type, &ret_type) {
                            self.diagnostics.add(KivError::type_error(
                                &ret_expr.span,
                                format!(
                                    "return type mismatch: expected {}, found {}",
                                    expected_type.name(),
                                    ret_type.name()
                                ),
                                "incompatible return type",
                                None,
                                kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                            ));
                        }
                    }
                }
            }

            HirStmtKind::While { condition, body } => {
                *condition = self.check_expr(condition.clone());

                // Condition must be boolean
                let cond_type = self.infer_expr_type(condition);
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.diagnostics.add(KivError::type_error(
                        &condition.span,
                        format!("while condition must be Bool, found {}", cond_type.name()),
                        "expected Bool",
                        None,
                        kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                    ));
                }

                *body = self.check_block(body.clone());
            }

            HirStmtKind::For {
                var_id,
                iterable,
                body,
                ..
            } => {
                *iterable = self.check_expr(iterable.clone());

                // TODO: Check that iterable is actually iterable
                // For now, assume it's valid and the variable has Unknown type
                self.var_types.insert(*var_id, Type::Unknown);
                self.var_mutability.insert(*var_id, false);

                *body = self.check_block(body.clone());
            }

            HirStmtKind::Break | HirStmtKind::Continue => {
                // TODO: Verify we're inside a loop
                // For now, accept these statements
            }

            HirStmtKind::Expr { expr } => {
                *expr = self.check_expr(expr.clone());
            }
        }

        stmt
    }
}
