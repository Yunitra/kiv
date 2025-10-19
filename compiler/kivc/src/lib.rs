//! # Kiv Compiler Library
//!
//! This crate provides the core compilation pipeline for the Kiv programming language.
//! It integrates all compiler stages from lexing to MIR generation.

pub mod compiler;
pub mod config;
pub mod session;

// Re-exports for convenience
pub use compiler::{compile_source, compile_to_ast, compile_to_hir, compile_to_mir, CompileResult};
pub use config::{CompilerConfig, OptLevel, StopAfter};
pub use session::CompilerSession;

// Re-export diagnostic types
pub use kivc_diagnostics::{DiagnosticsCollector, KivError};

// Re-export AST types
pub use kivc_ast::Program as AstProgram;

// Re-export HIR types
pub use kivc_hir::HirProgram;

// Re-export MIR types
pub use kivc_mir::MirProgram;
