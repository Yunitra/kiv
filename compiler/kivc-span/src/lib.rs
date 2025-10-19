//! Span and source location management for the Kiv compiler.
//!
//! This crate provides types for tracking source code locations and spans,
//! enabling accurate error reporting and diagnostics.

mod source_file;
mod span;

pub use source_file::SourceFile;
pub use span::Span;
pub use text_size::{TextRange, TextSize};
