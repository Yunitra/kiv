//! Expression formatting.

use super::Formatter;
use kivc_ast::{BinOp, Expr, ExprKind};
use std::fmt::Write as FmtWrite;

impl Formatter {
    /// Formats an expression
    pub(super) fn format_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(lit) => {
                self.format_literal(lit);
            }

            ExprKind::Binary { op, lhs, rhs } => {
                self.format_expr(lhs);
                self.output.push(' ');
                self.format_binop(*op);
                self.output.push(' ');
                self.format_expr(rhs);
            }

            ExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.output.push_str("if ");
                self.format_expr(cond);
                self.output.push_str(" {\n");

                self.indent_level += 1;
                self.format_block(then_branch);
                self.indent_level -= 1;

                self.write_indent();
                self.output.push('}');

                if let Some(else_branch) = else_branch {
                    self.output.push_str(" else {\n");

                    self.indent_level += 1;
                    self.format_block(else_branch);
                    self.indent_level -= 1;

                    self.write_indent();
                    self.output.push('}');
                }
            }

            ExprKind::Var { name } => {
                self.output.push_str(name);
            }

            ExprKind::Call { func, args } => {
                write!(self.output, "{}(", func).unwrap();

                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expr(arg);
                }

                self.output.push(')');
            }

            ExprKind::Assign { target, value } => {
                write!(self.output, "{} = ", target).unwrap();
                self.format_expr(value);
            }

            ExprKind::Match { value, arms } => {
                self.output.push_str("match ");
                self.format_expr(value);
                self.output.push_str(" {\n");
                self.indent_level += 1;

                for arm in arms {
                    self.write_indent();
                    self.format_pattern(&arm.pattern);
                    self.output.push_str(" => ");
                    self.format_expr(&arm.body);
                    self.output.push_str(",\n");
                }

                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');
            }

            ExprKind::Block(block) => {
                self.output.push_str("{\n");
                self.indent_level += 1;
                self.format_block(block);
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');
            }
        }
    }

    /// Formats a binary operator
    pub(super) fn format_binop(&mut self, op: BinOp) {
        let op_str = match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Eq => "==",
            BinOp::NotEq => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
        };
        self.output.push_str(op_str);
    }
}
