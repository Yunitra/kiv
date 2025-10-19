//! Expression type checking.

use super::checker::TypeChecker;
use crate::types::Type;
use kivc_diagnostics::KivError;
use kivc_hir::{HirExpr, HirExprKind};

impl TypeChecker {
    pub(super) fn check_expr(&mut self, mut expr: HirExpr) -> HirExpr {
        match &mut expr.kind {
            HirExprKind::Literal(_) => {
                // Literals are already correctly typed
            }

            HirExprKind::Var { var_id, name } => {
                // Ensure variable is declared
                if !self.var_types.contains_key(var_id) {
                    self.diagnostics.add(KivError::syntax(
                        &expr.span,
                        format!("undefined variable '{}'", name),
                        "not found",
                        None,
                        kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                    ));
                }
            }

            HirExprKind::Binary { op, lhs, rhs } => {
                **lhs = self.check_expr(*lhs.clone());
                **rhs = self.check_expr(*rhs.clone());

                let lhs_type = self.infer_expr_type(lhs);
                let rhs_type = self.infer_expr_type(rhs);

                // Check if operation is valid for these types
                if let Some(result_type) = self.ctx.binary_op_result_type(*op, &lhs_type, &rhs_type)
                {
                    // Update expression type
                    expr.ty = self.type_to_type_id(&result_type);
                } else {
                    self.diagnostics.add(KivError::type_error(
                        &expr.span,
                        format!(
                            "cannot apply operator to {} and {}",
                            lhs_type.name(),
                            rhs_type.name()
                        ),
                        "incompatible types for operation",
                        None,
                        kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                    ));
                }
            }

            HirExprKind::Call { args, .. } => {
                // Check argument types
                for arg in args {
                    *arg = self.check_expr(arg.clone());
                }
                // Function type checking would happen in a later phase
            }

            HirExprKind::Assign {
                var_id,
                target,
                value,
            } => {
                **value = self.check_expr(*value.clone());

                // Check if variable is mutable
                if let Some(&mutable) = self.var_mutability.get(var_id)
                    && !mutable
                {
                    self.diagnostics.add(KivError::syntax(
                        &expr.span,
                        format!("cannot assign to immutable variable '{}'", target),
                        "not mutable",
                        Some("consider making it mutable with 'mut'".to_string()),
                        kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                    ));
                }

                // Check type compatibility
                if let Some(var_type) = self.var_types.get(var_id) {
                    let value_type = self.infer_expr_type(value);

                    if !self.ctx.are_compatible(var_type, &value_type) {
                        self.diagnostics.add(KivError::type_error(
                            &value.span,
                            format!(
                                "type mismatch in assignment: expected {}, found {}",
                                var_type.name(),
                                value_type.name()
                            ),
                            "incompatible type",
                            None,
                            kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                        ));
                    }
                }
            }

            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                **cond = self.check_expr(*cond.clone());

                // Condition must be boolean
                let cond_type = self.infer_expr_type(cond);
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.diagnostics.add(KivError::type_error(
                        &cond.span,
                        format!("if condition must be Bool, found {}", cond_type.name()),
                        "expected Bool",
                        None,
                        kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                    ));
                }

                *then_branch = self.check_block(then_branch.clone());

                if let Some(else_b) = else_branch {
                    *else_b = self.check_block(else_b.clone());
                }
            }

            HirExprKind::Match { value, arms } => {
                **value = self.check_expr(*value.clone());

                // Check all arms
                for arm in arms {
                    arm.body = self.check_expr(arm.body.clone());
                    // TODO: Check pattern exhaustiveness
                    // TODO: Check all arms return compatible types
                }
            }

            HirExprKind::Block(block) => {
                *block = self.check_block(block.clone());
            }
        }

        expr
    }
}
