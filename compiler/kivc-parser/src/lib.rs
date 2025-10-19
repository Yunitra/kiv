//! A recursive descent parser for the Kiv programming language.
//!
//! This crate provides the `Parser` struct, which takes a stream of tokens
//! from the lexer and produces an Abstract Syntax Tree (AST). It uses:
//! - Recursive descent parsing for statements and declarations
//! - Pratt parsing for expressions (operator precedence)
//! - Error recovery at synchronization points (`;`, `}`, `fun`)

#![allow(clippy::result_large_err)]

mod expr;
mod item;
mod parser;
mod recovery;
mod stmt;
mod ty;

pub use kivc_ast::Program;
pub use parser::Parser;
