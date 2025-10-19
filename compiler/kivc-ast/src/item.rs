use crate::stmt::Block;
use crate::ty::Type;
use kivc_span::Span;

/// A function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

impl Param {
    /// Creates a new parameter
    pub fn new(name: String, ty: Type, span: Span) -> Self {
        Self { name, ty, span }
    }
}

/// A function definition
#[derive(Debug, Clone)]
pub struct FunDef {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Option<Type>,
    pub body: Block,
    pub span: Span,
}

impl FunDef {
    /// Creates a new function definition
    pub fn new(
        name: String,
        params: Vec<Param>,
        ret_ty: Option<Type>,
        body: Block,
        span: Span,
    ) -> Self {
        Self {
            name,
            params,
            ret_ty,
            body,
            span,
        }
    }
}

/// A complete program (collection of function definitions)
#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<FunDef>,
}

impl Program {
    /// Creates a new program
    pub fn new(functions: Vec<FunDef>) -> Self {
        Self { functions }
    }

    /// Creates an empty program
    pub fn empty() -> Self {
        Self {
            functions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{Expr, Literal};
    use crate::stmt::Stmt;
    use crate::ty::TypeKind;
    use kivc_span::SourceFile;

    fn dummy_span() -> Span {
        let file = SourceFile::new("test.kiv".to_string(), "test".to_string());
        Span::new(file, 0.into(), 4.into())
    }

    #[test]
    fn test_param_creation() {
        let ty = Type::new(TypeKind::Int, dummy_span());
        let param = Param::new("x".to_string(), ty, dummy_span());

        assert_eq!(param.name, "x");
    }

    #[test]
    fn test_fundef_creation() {
        let ty = Type::new(TypeKind::Int, dummy_span());
        let param = Param::new("x".to_string(), ty.clone(), dummy_span());

        let expr = Expr::literal(Literal::Int(42), dummy_span());
        let stmt = Stmt::return_stmt(Some(expr), dummy_span());
        let body = Block::new(vec![stmt], dummy_span());

        let fundef = FunDef::new(
            "test".to_string(),
            vec![param],
            Some(ty),
            body,
            dummy_span(),
        );

        assert_eq!(fundef.name, "test");
        assert_eq!(fundef.params.len(), 1);
        assert!(fundef.ret_ty.is_some());
    }

    #[test]
    fn test_program_creation() {
        let program = Program::new(vec![]);
        assert_eq!(program.functions.len(), 0);

        let empty = Program::empty();
        assert_eq!(empty.functions.len(), 0);
    }
}
