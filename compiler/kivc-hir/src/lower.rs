//! Lowering from AST to HIR.

use crate::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunDef, HirParam, HirProgram, HirStmt, HirStmtKind,
};
use crate::id::{FunId, TypeId, VarId};
use crate::scope::ScopeStack;
use kivc_ast::{Block, Expr, ExprKind, FunDef, Program, Stmt, StmtKind, TypeKind};
use kivc_diagnostics::{DiagnosticsCollector, KivError};

/// Lowers an AST program to HIR
pub fn lower_program(program: Program) -> Result<HirProgram, DiagnosticsCollector> {
    let mut lowerer = Lowerer::new();
    let hir_program = lowerer.lower_program(program);

    if lowerer.diagnostics.has_errors() {
        Err(lowerer.diagnostics)
    } else {
        Ok(hir_program)
    }
}

/// The lowering context
struct Lowerer {
    scopes: ScopeStack,
    diagnostics: DiagnosticsCollector,
    next_fun_id: usize,
}

impl Lowerer {
    fn new() -> Self {
        Self {
            scopes: ScopeStack::new(),
            diagnostics: DiagnosticsCollector::new(),
            next_fun_id: 0,
        }
    }

    fn next_fun_id(&mut self) -> FunId {
        let id = FunId::new(self.next_fun_id);
        self.next_fun_id += 1;
        id
    }

    fn lower_program(&mut self, program: Program) -> HirProgram {
        let functions = program
            .functions
            .into_iter()
            .map(|f| self.lower_function(f))
            .collect();

        HirProgram { functions }
    }

    fn lower_function(&mut self, fun: FunDef) -> HirFunDef {
        let fun_id = self.next_fun_id();

        // Create a new scope for the function
        self.scopes.push_scope();

        // Lower parameters
        let params = fun
            .params
            .into_iter()
            .map(|p| {
                let var_id = self.scopes.declare(p.name.clone());
                let ty = self.resolve_type(&p.ty);
                HirParam {
                    var_id,
                    name: p.name,
                    ty,
                    span: p.span,
                }
            })
            .collect();

        // Lower return type
        let return_type = fun.ret_ty.as_ref().map(|ty| self.resolve_type(ty));

        // Lower body
        let body = self.lower_block(fun.body);

        // Pop function scope
        self.scopes.pop_scope();

        HirFunDef {
            id: fun_id,
            name: fun.name,
            params,
            return_type,
            body,
            span: fun.span,
        }
    }

    fn lower_block(&mut self, block: Block) -> HirBlock {
        self.scopes.push_scope();

        let stmts = block
            .stmts
            .into_iter()
            .map(|s| self.lower_stmt(s))
            .collect();

        self.scopes.pop_scope();

        HirBlock {
            stmts,
            span: block.span,
        }
    }

    fn lower_stmt(&mut self, stmt: Stmt) -> HirStmt {
        let span = stmt.span.clone();

        let kind = match stmt.kind {
            StmtKind::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                // Check for redeclaration in current scope
                if self.scopes.is_declared_in_current(&name) {
                    self.diagnostics.add(KivError::syntax(
                        &span,
                        format!("variable '{}' is already declared in this scope", name),
                        "redeclared here",
                        Some("use a different name or remove the previous declaration".to_string()),
                        kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                    ));
                }

                let var_id = self.scopes.declare(name.clone());
                let resolved_ty = ty.as_ref().map(|t| self.resolve_type(t));
                let init_expr = self.lower_expr(init);

                HirStmtKind::Let {
                    var_id,
                    name,
                    mutable,
                    ty: resolved_ty,
                    init: init_expr,
                }
            }

            StmtKind::Const { name, value } => {
                if self.scopes.is_declared_in_current(&name) {
                    self.diagnostics.add(KivError::syntax(
                        &span,
                        format!("constant '{}' is already declared in this scope", name),
                        "redeclared here",
                        None,
                        kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                    ));
                }

                let var_id = self.scopes.declare(name.clone());
                let value_expr = self.lower_expr(value);

                HirStmtKind::Const {
                    var_id,
                    name,
                    value: value_expr,
                }
            }

            StmtKind::Return { value } => HirStmtKind::Return {
                value: value.map(|v| self.lower_expr(v)),
            },

            StmtKind::Expr { expr } => HirStmtKind::Expr {
                expr: self.lower_expr(expr),
            },
        };

        HirStmt::new(kind, span)
    }

    fn lower_expr(&mut self, expr: Expr) -> HirExpr {
        let span = expr.span.clone();

        let kind = match expr.kind {
            ExprKind::Literal(lit) => HirExprKind::Literal(lit),

            ExprKind::Var { name } => {
                match self.scopes.lookup(&name) {
                    Some(var_id) => HirExprKind::Var {
                        var_id,
                        name: name.clone(),
                    },
                    None => {
                        self.diagnostics.add(KivError::syntax(
                            &span,
                            format!("undefined variable '{}'", name),
                            "not found in this scope",
                            Some("make sure the variable is declared before use".to_string()),
                            kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                        ));

                        // Return a dummy var to continue lowering
                        HirExprKind::Var {
                            var_id: VarId::new(0),
                            name,
                        }
                    }
                }
            }

            ExprKind::Binary { op, lhs, rhs } => {
                let lhs_hir = self.lower_expr(*lhs);
                let rhs_hir = self.lower_expr(*rhs);

                HirExprKind::Binary {
                    op,
                    lhs: Box::new(lhs_hir),
                    rhs: Box::new(rhs_hir),
                }
            }

            ExprKind::Call { func, args } => {
                let args_hir = args.into_iter().map(|a| self.lower_expr(a)).collect();

                HirExprKind::Call {
                    func,
                    args: args_hir,
                }
            }

            ExprKind::Assign { target, value } => {
                let var_id = match self.scopes.lookup(&target) {
                    Some(id) => id,
                    None => {
                        self.diagnostics.add(KivError::syntax(
                            &span,
                            format!("cannot assign to undefined variable '{}'", target),
                            "not found in this scope",
                            None,
                            kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
                        ));
                        VarId::new(0)
                    }
                };

                let value_hir = self.lower_expr(*value);

                HirExprKind::Assign {
                    var_id,
                    target,
                    value: Box::new(value_hir),
                }
            }

            ExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_hir = self.lower_expr(*cond);
                let then_hir = self.lower_block(then_branch);
                let else_hir = else_branch.map(|b| self.lower_block(b));

                HirExprKind::If {
                    cond: Box::new(cond_hir),
                    then_branch: then_hir,
                    else_branch: else_hir,
                }
            }
        };

        HirExpr::new(kind, span)
    }

    fn resolve_type(&self, ty: &kivc_ast::Type) -> TypeId {
        // For MVP, we use simple type IDs based on TypeKind
        match ty.kind {
            TypeKind::Int => TypeId::new(0),
            TypeKind::Float => TypeId::new(1),
            TypeKind::Bool => TypeId::new(2),
            TypeKind::Text => TypeId::new(3),
            TypeKind::Unit => TypeId::new(4),
        }
    }
}
