//! Literal and pattern formatting.

use super::Formatter;
use kivc_ast::{Literal, Pattern};
use std::fmt::Write as FmtWrite;

impl Formatter {
    /// Formats a literal
    pub(super) fn format_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Int(n) => write!(self.output, "{}", n).unwrap(),
            Literal::Float(f) => write!(self.output, "{}", f).unwrap(),
            Literal::Bool(b) => self.output.push_str(if *b { "true" } else { "false" }),
            Literal::Text(s) => write!(self.output, "\"{}\"", escape_string(s)).unwrap(),
            Literal::Unit => self.output.push_str("()"),
        }
    }

    /// Formats a pattern
    pub(super) fn format_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard => self.output.push('_'),
            Pattern::Literal(lit) => self.format_literal(lit),
            Pattern::Binding(name) => self.output.push_str(name),
            Pattern::Or(patterns) => {
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" | ");
                    }
                    self.format_pattern(p);
                }
            }
        }
    }
}

/// Escapes a string for output
fn escape_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            '\n' => vec!['\\', 'n'],
            '\r' => vec!['\\', 'r'],
            '\t' => vec!['\\', 't'],
            c => vec![c],
        })
        .collect()
}
