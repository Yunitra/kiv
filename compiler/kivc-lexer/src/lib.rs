//! A hand-written lexical analyzer for the Kiv programming language.
//!
//! This crate provides the `Lexer` struct, which takes a source file and
//! produces a stream of `Token`s. It handles whitespace, comments,
//! keywords, identifiers, literals, and operators, and reports lexical errors
//! using the `kivc-diagnostics` system.

mod intern;
mod lexer;
mod token;

pub use intern::{InternedString, Interner};
pub use lexer::Lexer;
pub use token::{Token, TokenKind};
