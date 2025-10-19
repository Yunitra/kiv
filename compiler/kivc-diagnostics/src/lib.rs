//! Diagnostic and error reporting system for the Kiv compiler.
//!
//! This crate provides structured error types with rich formatting capabilities
//! powered by miette, including source code snippets, labels, and helpful suggestions.

mod collector;
mod error;

pub mod error_code;

pub use collector::DiagnosticsCollector;
pub use error::KivError;
pub use error_code::ErrorCode;

// Re-export commonly used types from miette
pub use miette::{Diagnostic, Report, Result};
