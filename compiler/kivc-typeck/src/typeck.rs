//! Type checking implementation.

use crate::types::{Type, TypeContext};
use kivc_diagnostics::{DiagnosticsCollector, KivError};
use kivc_hir::{
    HirBlock, HirExpr, HirExprKind, HirFunDef, HirProgram, HirStmt, HirStmtKind, TypeId, VarId,
};
use std::collections::HashMap;

/// Type checks a HIR program
pub fn typecheck_program(program: HirProgram) -> Result<HirProgram, DiagnosticsCollector> {
    let mut checker = TypeChecker::new();
    let checked_program = checker.check_program(program);

    if checker.diagnostics.has_errors() {
        Err(checker.diagnostics)
    } else {
        Ok(checked_program)
    }
}

/// The type checking context
struct TypeChecker {
    ctx: TypeContext,
    var_types: HashMap<VarId, Type>,
    var_mutability: HashMap<VarId, bool>,
    current_return_type: Option<TypeId>,
    diagnostics: DiagnosticsCollector,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            ctx: TypeContext::new(),
            var_types: HashMap::new(),
            var_mutability: HashMap::new(),
            current_return_type: None,
            diagnostics: DiagnosticsCollector::new(),
        }
    }

    fn check_program(&mut self, program: HirProgram) -> HirProgram {
        let functions = program
            .functions
            .into_iter()
            .map(|f| self.check_function(f))
            .collect();

        HirProgram { functions }
    }

    fn check_function(&mut self, mut fun: HirFunDef) -> HirFunDef {
        // Store parameter types
        for param in &fun.params {
            if let Some(ty) = self.ctx.get(param.ty) {
                self.var_types.insert(param.var_id, ty.clone());
                self.var_mutability.insert(param.var_id, false);
            }
        }

        // Store return type for validation
        self.current_return_type = fun.return_type;

        // Check function body
        fun.body = self.check_block(fun.body);

        // Clear function-specific state
        self.var_types.clear();
        self.var_mutability.clear();
        self.current_return_type = None;

        fun
    }

    fn check_block(&mut self, mut block: HirBlock) -> HirBlock {
        block.stmts = block
            .stmts
            .into_iter()
            .map(|s| self.check_stmt(s))
            .collect();

        block
    }

    fn check_stmt(&mut self, mut stmt: HirStmt) -> HirStmt {
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

            HirStmtKind::Expr { expr } => {
                *expr = self.check_expr(expr.clone());
            }
        }

        stmt
    }

    fn check_expr(&mut self, mut expr: HirExpr) -> HirExpr {
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
        }

        expr
    }

    fn infer_expr_type(&self, expr: &HirExpr) -> Type {
        // If type is already set, use it
        if let Some(ty_id) = expr.ty {
            return self.ctx.get(ty_id).cloned().unwrap_or(Type::Unknown);
        }

        match &expr.kind {
            HirExprKind::Literal(lit) => match lit {
                kivc_ast::Literal::Int(_) => Type::Int,
                kivc_ast::Literal::Float(_) => Type::Float,
                kivc_ast::Literal::Bool(_) => Type::Bool,
                kivc_ast::Literal::Text(_) => Type::Text,
                kivc_ast::Literal::Unit => Type::Unit,
            },

            HirExprKind::Var { var_id, .. } => {
                self.var_types.get(var_id).cloned().unwrap_or(Type::Unknown)
            }

            HirExprKind::Binary { op, lhs, rhs } => {
                let lhs_type = self.infer_expr_type(lhs);
                let rhs_type = self.infer_expr_type(rhs);

                self.ctx
                    .binary_op_result_type(*op, &lhs_type, &rhs_type)
                    .unwrap_or(Type::Unknown)
            }

            HirExprKind::Call { .. } => Type::Unknown,

            HirExprKind::Assign { .. } => Type::Unit,

            HirExprKind::If { .. } => Type::Unknown,
        }
    }

    fn type_to_type_id(&self, ty: &Type) -> Option<TypeId> {
        match ty {
            Type::Int => Some(TypeId::new(0)),
            Type::Float => Some(TypeId::new(1)),
            Type::Bool => Some(TypeId::new(2)),
            Type::Text => Some(TypeId::new(3)),
            Type::Unit => Some(TypeId::new(4)),
            Type::Unknown => None,
        }
    }
}
