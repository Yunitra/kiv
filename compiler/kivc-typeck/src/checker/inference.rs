//! Type inference.

use super::context::TypeChecker;
use crate::types::Type;
use kivc_hir::{HirExpr, HirExprKind, TypeId};

impl TypeChecker {
    pub(super) fn infer_expr_type(&self, expr: &HirExpr) -> Type {
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

            HirExprKind::Match { .. } => Type::Unknown,

            HirExprKind::Block(_) => Type::Unknown,
        }
    }

    pub(super) fn type_to_type_id(&self, ty: &Type) -> Option<TypeId> {
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
