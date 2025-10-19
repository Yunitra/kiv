//! HIR node definitions.

use crate::id::{FunId, TypeId, VarId};
use kivc_ast::{BinOp, Literal};
use kivc_span::Span;

/// A complete HIR program
#[derive(Debug, Clone)]
pub struct HirProgram {
    pub functions: Vec<HirFunDef>,
}

/// A function definition in HIR
#[derive(Debug, Clone)]
pub struct HirFunDef {
    pub id: FunId,
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: Option<TypeId>,
    pub body: HirBlock,
    pub span: Span,
}

/// A function parameter
#[derive(Debug, Clone)]
pub struct HirParam {
    pub var_id: VarId,
    pub name: String,
    pub ty: TypeId,
    pub span: Span,
}

/// A block of statements
#[derive(Debug, Clone)]
pub struct HirBlock {
    pub stmts: Vec<HirStmt>,
    pub span: Span,
}

/// A statement in HIR
#[derive(Debug, Clone)]
pub struct HirStmt {
    pub kind: HirStmtKind,
    pub span: Span,
}

/// The kind of statement
#[derive(Debug, Clone)]
pub enum HirStmtKind {
    /// Variable declaration
    Let {
        var_id: VarId,
        name: String,
        mutable: bool,
        ty: Option<TypeId>,
        init: HirExpr,
    },

    /// Constant declaration
    Const {
        var_id: VarId,
        name: String,
        value: HirExpr,
    },

    /// Return statement
    Return { value: Option<HirExpr> },

    /// While loop
    While { condition: HirExpr, body: HirBlock },

    /// For loop
    For {
        var_id: VarId,
        variable: String,
        iterable: HirExpr,
        body: HirBlock,
    },

    /// Break statement
    Break,

    /// Continue statement
    Continue,

    /// Expression statement
    Expr { expr: HirExpr },
}

/// An expression in HIR
#[derive(Debug, Clone)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub ty: Option<TypeId>,
    pub span: Span,
}

/// The kind of expression
#[derive(Debug, Clone)]
pub enum HirExprKind {
    /// A literal value
    Literal(Literal),

    /// Variable reference
    Var { var_id: VarId, name: String },

    /// Binary operation
    Binary {
        op: BinOp,
        lhs: Box<HirExpr>,
        rhs: Box<HirExpr>,
    },

    /// Function call
    Call { func: String, args: Vec<HirExpr> },

    /// Assignment
    Assign {
        var_id: VarId,
        target: String,
        value: Box<HirExpr>,
    },

    /// If expression
    If {
        cond: Box<HirExpr>,
        then_branch: HirBlock,
        else_branch: Option<HirBlock>,
    },

    /// Match expression
    Match {
        value: Box<HirExpr>,
        arms: Vec<HirMatchArm>,
    },

    /// Block expression
    Block(HirBlock),
}

/// A match arm in HIR
#[derive(Debug, Clone)]
pub struct HirMatchArm {
    pub pattern: HirPattern,
    pub body: HirExpr,
    pub span: Span,
}

/// A pattern in HIR
#[derive(Debug, Clone)]
pub enum HirPattern {
    Wildcard,
    Literal(Literal),
    Binding { var_id: VarId, name: String },
    Or(Vec<HirPattern>),
}

impl HirExpr {
    /// Creates a new HIR expression with no type information
    pub fn new(kind: HirExprKind, span: Span) -> Self {
        Self {
            kind,
            ty: None,
            span,
        }
    }

    /// Creates a new HIR expression with type information
    pub fn with_type(kind: HirExprKind, ty: TypeId, span: Span) -> Self {
        Self {
            kind,
            ty: Some(ty),
            span,
        }
    }
}

impl HirStmt {
    /// Creates a new HIR statement
    pub fn new(kind: HirStmtKind, span: Span) -> Self {
        Self { kind, span }
    }
}
