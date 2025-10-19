use crate::expr::Expr;
use crate::ty::Type;
use kivc_span::Span;

/// A block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

impl Block {
    /// Creates a new block
    pub fn new(stmts: Vec<Stmt>, span: Span) -> Self {
        Self { stmts, span }
    }
}

/// A statement in the AST
#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

/// The kind of statement
#[derive(Debug, Clone)]
pub enum StmtKind {
    /// Variable declaration: let [mut] name: type = init;
    Let {
        mutable: bool,
        name: String,
        ty: Option<Type>,
        init: Expr,
    },

    /// Constant declaration: const name = value;
    Const { name: String, value: Expr },

    /// Return statement: return [value];
    Return { value: Option<Expr> },

    /// While loop: while condition { body }
    While { condition: Expr, body: Block },

    /// For loop: for variable in iterable { body }
    For {
        variable: String,
        iterable: Expr,
        body: Block,
    },

    /// Break statement: break;
    Break,

    /// Continue statement: continue;
    Continue,

    /// Expression statement
    Expr { expr: Expr },
}

impl Stmt {
    /// Creates a new statement
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Creates a let statement
    pub fn let_stmt(mutable: bool, name: String, ty: Option<Type>, init: Expr, span: Span) -> Self {
        Self::new(
            StmtKind::Let {
                mutable,
                name,
                ty,
                init,
            },
            span,
        )
    }

    /// Creates a const statement
    pub fn const_stmt(name: String, value: Expr, span: Span) -> Self {
        Self::new(StmtKind::Const { name, value }, span)
    }

    /// Creates a return statement
    pub fn return_stmt(value: Option<Expr>, span: Span) -> Self {
        Self::new(StmtKind::Return { value }, span)
    }

    /// Creates an expression statement
    pub fn expr_stmt(expr: Expr, span: Span) -> Self {
        Self::new(StmtKind::Expr { expr }, span)
    }

    /// Creates a while statement
    pub fn while_stmt(condition: Expr, body: Block, span: Span) -> Self {
        Self::new(StmtKind::While { condition, body }, span)
    }

    /// Creates a for statement
    pub fn for_stmt(variable: String, iterable: Expr, body: Block, span: Span) -> Self {
        Self::new(
            StmtKind::For {
                variable,
                iterable,
                body,
            },
            span,
        )
    }

    /// Creates a break statement
    pub fn break_stmt(span: Span) -> Self {
        Self::new(StmtKind::Break, span)
    }

    /// Creates a continue statement
    pub fn continue_stmt(span: Span) -> Self {
        Self::new(StmtKind::Continue, span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{Expr, Literal};
    use kivc_span::SourceFile;

    fn dummy_span() -> Span {
        let file = SourceFile::new("test.kiv".to_string(), "test".to_string());
        Span::new(file, 0.into(), 4.into())
    }

    #[test]
    fn test_let_stmt() {
        let init = Expr::literal(Literal::Int(42), dummy_span());
        let stmt = Stmt::let_stmt(false, "x".to_string(), None, init, dummy_span());

        assert!(matches!(stmt.kind, StmtKind::Let { .. }));
    }

    #[test]
    fn test_const_stmt() {
        let value = Expr::literal(Literal::Int(42), dummy_span());
        let stmt = Stmt::const_stmt("PI".to_string(), value, dummy_span());

        assert!(matches!(stmt.kind, StmtKind::Const { .. }));
    }

    #[test]
    fn test_return_stmt() {
        let value = Expr::literal(Literal::Int(42), dummy_span());
        let stmt = Stmt::return_stmt(Some(value), dummy_span());

        assert!(matches!(stmt.kind, StmtKind::Return { .. }));
    }

    #[test]
    fn test_expr_stmt() {
        let expr = Expr::literal(Literal::Int(42), dummy_span());
        let stmt = Stmt::expr_stmt(expr, dummy_span());

        assert!(matches!(stmt.kind, StmtKind::Expr { .. }));
    }

    #[test]
    fn test_block() {
        let expr = Expr::literal(Literal::Int(42), dummy_span());
        let stmt = Stmt::expr_stmt(expr, dummy_span());
        let block = Block::new(vec![stmt], dummy_span());

        assert_eq!(block.stmts.len(), 1);
    }
}
