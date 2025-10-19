//! Type system definitions.

use kivc_hir::TypeId;
use std::collections::HashMap;

/// The type of a value in Kiv
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    Bool,
    Text,
    Unit,
    /// Type inference placeholder
    Unknown,
}

/// A context holding all type information
#[derive(Debug, Clone)]
pub struct TypeContext {
    types: HashMap<TypeId, Type>,
}

impl TypeContext {
    /// Creates a new type context with built-in types
    pub fn new() -> Self {
        let mut types = HashMap::new();

        // Register built-in types with their IDs
        types.insert(TypeId::new(0), Type::Int);
        types.insert(TypeId::new(1), Type::Float);
        types.insert(TypeId::new(2), Type::Bool);
        types.insert(TypeId::new(3), Type::Text);
        types.insert(TypeId::new(4), Type::Unit);

        Self { types }
    }

    /// Gets the type associated with a TypeId
    pub fn get(&self, id: TypeId) -> Option<&Type> {
        self.types.get(&id)
    }

    /// Checks if two types are compatible
    pub fn are_compatible(&self, left: &Type, right: &Type) -> bool {
        match (left, right) {
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (a, b) => a == b,
        }
    }

    /// Gets the result type of a binary operation
    pub fn binary_op_result_type(
        &self,
        op: kivc_ast::BinOp,
        left: &Type,
        right: &Type,
    ) -> Option<Type> {
        use kivc_ast::BinOp;

        match op {
            // Arithmetic operators: require numeric types, return same type
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                if !self.are_compatible(left, right) {
                    return None;
                }

                match left {
                    Type::Int | Type::Float => Some(left.clone()),
                    Type::Unknown => Some(Type::Unknown),
                    _ => None,
                }
            }

            // Comparison operators: require compatible types, return bool
            BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                if !self.are_compatible(left, right) {
                    return None;
                }

                match left {
                    Type::Int | Type::Float | Type::Bool | Type::Unknown => Some(Type::Bool),
                    _ => None,
                }
            }
        }
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Type {
    /// Returns a human-readable name for the type
    pub fn name(&self) -> &'static str {
        match self {
            Type::Int => "Int",
            Type::Float => "Float",
            Type::Bool => "Bool",
            Type::Text => "Text",
            Type::Unit => "Unit",
            Type::Unknown => "<unknown>",
        }
    }

    /// Checks if this type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_context() {
        let ctx = TypeContext::new();

        assert_eq!(ctx.get(TypeId::new(0)), Some(&Type::Int));
        assert_eq!(ctx.get(TypeId::new(1)), Some(&Type::Float));
        assert_eq!(ctx.get(TypeId::new(2)), Some(&Type::Bool));
    }

    #[test]
    fn test_type_compatibility() {
        let ctx = TypeContext::new();

        assert!(ctx.are_compatible(&Type::Int, &Type::Int));
        assert!(!ctx.are_compatible(&Type::Int, &Type::Float));
        assert!(ctx.are_compatible(&Type::Unknown, &Type::Int));
    }

    #[test]
    fn test_binary_op_result_type() {
        let ctx = TypeContext::new();

        // Arithmetic
        assert_eq!(
            ctx.binary_op_result_type(kivc_ast::BinOp::Add, &Type::Int, &Type::Int),
            Some(Type::Int)
        );

        // Comparison
        assert_eq!(
            ctx.binary_op_result_type(kivc_ast::BinOp::Eq, &Type::Int, &Type::Int),
            Some(Type::Bool)
        );

        // Incompatible types
        assert_eq!(
            ctx.binary_op_result_type(kivc_ast::BinOp::Add, &Type::Int, &Type::Bool),
            None
        );
    }
}
