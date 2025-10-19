use crate::stmt::Block;
use kivc_span::Span;

/// A literal value
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Text(String),
    Unit,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /

    // Comparison
    Eq,    // ==
    NotEq, // !=
    Lt,    // <
    Le,    // <=
    Gt,    // >
    Ge,    // >=
}

/// An expression in the AST
#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

/// The kind of expression
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// A literal value
    Literal(Literal),

    /// Binary operation: lhs op rhs
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    /// If expression: if cond { then_branch } else { else_branch }
    If {
        cond: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Block>,
    },

    /// Match expression: match value { patterns }
    Match {
        value: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// Variable reference
    Var { name: String },

    /// Function call
    Call { func: String, args: Vec<Expr> },

    /// Assignment
    Assign { target: String, value: Box<Expr> },

    /// Block expression: { stmts }
    Block(Block),
}

/// A match arm: pattern => body
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

/// A pattern in a match expression
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Wildcard pattern: _
    Wildcard,

    /// Literal pattern: 42, "hello", true, etc.
    Literal(Literal),

    /// Variable binding: x
    Binding(String),

    /// Or pattern: 1 | 2 | 3
    Or(Vec<Pattern>),
}

impl Expr {
    /// Creates a new expression
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Creates a literal expression
    pub fn literal(lit: Literal, span: Span) -> Self {
        Self::new(ExprKind::Literal(lit), span)
    }

    /// Creates a binary expression
    pub fn binary(op: BinOp, lhs: Expr, rhs: Expr, span: Span) -> Self {
        Self::new(
            ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            span,
        )
    }

    /// Creates a variable reference
    pub fn var(name: String, span: Span) -> Self {
        Self::new(ExprKind::Var { name }, span)
    }

    /// Creates a function call
    pub fn call(func: String, args: Vec<Expr>, span: Span) -> Self {
        Self::new(ExprKind::Call { func, args }, span)
    }

    /// Creates an assignment expression
    pub fn assign(target: String, value: Expr, span: Span) -> Self {
        Self::new(
            ExprKind::Assign {
                target,
                value: Box::new(value),
            },
            span,
        )
    }

    /// Creates an if expression
    pub fn if_expr(cond: Expr, then_branch: Block, else_branch: Option<Block>, span: Span) -> Self {
        Self::new(
            ExprKind::If {
                cond: Box::new(cond),
                then_branch,
                else_branch,
            },
            span,
        )
    }

    /// Creates a match expression
    pub fn match_expr(value: Expr, arms: Vec<MatchArm>, span: Span) -> Self {
        Self::new(
            ExprKind::Match {
                value: Box::new(value),
                arms,
            },
            span,
        )
    }

    /// Creates a block expression
    pub fn block_expr(block: Block, span: Span) -> Self {
        Self::new(ExprKind::Block(block), span)
    }
}

impl MatchArm {
    /// Creates a new match arm
    pub fn new(pattern: Pattern, body: Expr, span: Span) -> Self {
        Self {
            pattern,
            body,
            span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivc_span::SourceFile;

    fn dummy_span() -> Span {
        let file = SourceFile::new("test.kiv".to_string(), "test".to_string());
        Span::new(file, 0.into(), 4.into())
    }

    #[test]
    fn test_literal_int() {
        let expr = Expr::literal(Literal::Int(42), dummy_span());
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_literal_bool() {
        let expr = Expr::literal(Literal::Bool(true), dummy_span());
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Bool(true))));
    }

    #[test]
    fn test_variable() {
        let expr = Expr::var("x".to_string(), dummy_span());
        assert!(matches!(expr.kind, ExprKind::Var { .. }));
    }

    #[test]
    fn test_binary_expr() {
        let lhs = Expr::literal(Literal::Int(1), dummy_span());
        let rhs = Expr::literal(Literal::Int(2), dummy_span());
        let expr = Expr::binary(BinOp::Add, lhs, rhs, dummy_span());

        assert!(matches!(expr.kind, ExprKind::Binary { .. }));
    }

    #[test]
    fn test_call_expr() {
        let arg1 = Expr::literal(Literal::Int(1), dummy_span());
        let arg2 = Expr::literal(Literal::Int(2), dummy_span());
        let expr = Expr::call("add".to_string(), vec![arg1, arg2], dummy_span());

        assert!(matches!(expr.kind, ExprKind::Call { .. }));
    }

    #[test]
    fn test_assign_expr() {
        let value = Expr::literal(Literal::Int(42), dummy_span());
        let expr = Expr::assign("x".to_string(), value, dummy_span());

        assert!(matches!(expr.kind, ExprKind::Assign { .. }));
    }
}
