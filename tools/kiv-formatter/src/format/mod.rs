//! Code formatting implementation modules.
//!
//! This module is organized into focused sub-modules:
//! - `function`: Function and parameter formatting
//! - `stmt`: Statement formatting
//! - `expr`: Expression formatting
//! - `literal`: Literal and pattern formatting

mod expr;
mod function;
mod literal;
mod stmt;
mod tests;

use kivc_ast::*;

/// Formatter for Kiv code
pub struct Formatter {
    pub(super) output: String,
    pub(super) indent_level: usize,
    pub(super) indent_size: usize,
}

impl Formatter {
    /// Creates a new formatter
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_size: 4,
        }
    }

    /// Formats a program and returns the formatted code
    pub fn format_program(&mut self, program: &Program) -> String {
        for (i, func) in program.functions.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
            }
            self.format_function(func);
        }

        // Ensure file ends with a newline
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        self.output.clone()
    }

    /// Formats a block
    pub(super) fn format_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.format_stmt(stmt);
        }
    }

    /// Writes indentation
    pub(super) fn write_indent(&mut self) {
        for _ in 0..(self.indent_level * self.indent_size) {
            self.output.push(' ');
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}
