//! Statement formatting.

use super::Formatter;
use kivc_ast::{Stmt, StmtKind};
use std::fmt::Write as FmtWrite;

impl Formatter {
    /// Formats a statement
    pub(super) fn format_stmt(&mut self, stmt: &Stmt) {
        self.write_indent();

        match &stmt.kind {
            StmtKind::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                self.output.push_str("let ");
                if *mutable {
                    self.output.push_str("mut ");
                }
                self.output.push_str(name);

                if let Some(ty) = ty {
                    self.output.push_str(": ");
                    self.format_type(ty);
                }

                self.output.push_str(" = ");
                self.format_expr(init);
                self.output.push_str(";\n");
            }

            StmtKind::Const { name, value } => {
                write!(self.output, "const {} = ", name).unwrap();
                self.format_expr(value);
                self.output.push_str(";\n");
            }

            StmtKind::Return { value } => {
                self.output.push_str("return");
                if let Some(value) = value {
                    self.output.push(' ');
                    self.format_expr(value);
                }
                self.output.push_str(";\n");
            }

            StmtKind::While { condition, body } => {
                self.output.push_str("while ");
                self.format_expr(condition);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                self.format_block(body);
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("}\n");
            }

            StmtKind::For {
                variable,
                iterable,
                body,
            } => {
                self.output.push_str("for ");
                self.output.push_str(variable);
                self.output.push_str(" in ");
                self.format_expr(iterable);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                self.format_block(body);
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("}\n");
            }

            StmtKind::Break => {
                self.output.push_str("break;\n");
            }

            StmtKind::Continue => {
                self.output.push_str("continue;\n");
            }

            StmtKind::Expr { expr } => {
                self.format_expr(expr);
                self.output.push_str(";\n");
            }
        }
    }
}
