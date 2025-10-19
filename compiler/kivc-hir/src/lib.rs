//! High-level Intermediate Representation (HIR) for the Kiv compiler.
//!
//! HIR is a typed, desugared version of the AST where:
//! - Variable names are resolved to unique IDs
//! - Type information is attached to expressions (when available)
//! - Complex constructs are simplified (desugared)

mod hir;
mod id;
mod lower;
mod scope;

pub use hir::{HirBlock, HirExpr, HirExprKind, HirFunDef, HirProgram, HirStmt, HirStmtKind};
pub use id::{FunId, TypeId, VarId};
pub use lower::lower_program;
pub use scope::ScopeStack;
