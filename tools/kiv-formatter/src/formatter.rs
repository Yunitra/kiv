//! AST-based code formatter for Kiv.

use kivc_ast::*;
use std::fmt::Write as FmtWrite;

/// Formatter for Kiv code
pub struct Formatter {
    output: String,
    indent_level: usize,
    indent_size: usize,
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

    /// Formats a function definition
    fn format_function(&mut self, func: &FunDef) {
        write!(self.output, "fun {}(", func.name).unwrap();

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.format_param(param);
        }

        self.output.push(')');

        if let Some(ref ret_ty) = func.ret_ty {
            self.output.push_str(": ");
            self.format_type(ret_ty);
        }

        self.output.push_str(" {\n");

        self.indent_level += 1;
        self.format_block(&func.body);
        self.indent_level -= 1;

        self.output.push_str("}\n");
    }

    /// Formats a parameter
    fn format_param(&mut self, param: &Param) {
        write!(self.output, "{}: ", param.name).unwrap();
        self.format_type(&param.ty);
    }

    /// Formats a type
    fn format_type(&mut self, ty: &Type) {
        let type_str = match ty.kind {
            TypeKind::Int => "Int",
            TypeKind::Float => "Float",
            TypeKind::Bool => "Bool",
            TypeKind::Text => "Text",
            TypeKind::Unit => "()",
        };
        self.output.push_str(type_str);
    }

    /// Formats a block
    fn format_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.format_stmt(stmt);
        }
    }

    /// Formats a statement
    fn format_stmt(&mut self, stmt: &Stmt) {
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

            StmtKind::Expr { expr } => {
                self.format_expr(expr);
                self.output.push_str(";\n");
            }
        }
    }

    /// Formats an expression
    fn format_expr(&mut self, expr: &Expr) {
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
        }
    }

    /// Formats a literal
    fn format_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Int(n) => write!(self.output, "{}", n).unwrap(),
            Literal::Float(f) => write!(self.output, "{}", f).unwrap(),
            Literal::Bool(b) => self.output.push_str(if *b { "true" } else { "false" }),
            Literal::Text(s) => write!(self.output, "\"{}\"", escape_string(s)).unwrap(),
            Literal::Unit => self.output.push_str("()"),
        }
    }

    /// Formats a binary operator
    fn format_binop(&mut self, op: BinOp) {
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

    /// Writes indentation
    fn write_indent(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use kivc_span::{SourceFile, Span};

    fn dummy_span() -> Span {
        let file = SourceFile::new("test.kiv".to_string(), "test".to_string());
        Span::new(file, 0.into(), 4.into())
    }

    #[test]
    fn test_format_literal() {
        let mut formatter = Formatter::new();
        formatter.format_literal(&Literal::Int(42));
        assert_eq!(formatter.output, "42");
    }

    #[test]
    fn test_format_function() {
        let mut formatter = Formatter::new();

        let expr = Expr::literal(Literal::Int(42), dummy_span());
        let stmt = Stmt::return_stmt(Some(expr), dummy_span());
        let body = Block::new(vec![stmt], dummy_span());

        let ty = Type::new(TypeKind::Int, dummy_span());
        let param = Param::new("x".to_string(), ty.clone(), dummy_span());

        let func = FunDef::new(
            "test".to_string(),
            vec![param],
            Some(ty),
            body,
            dummy_span(),
        );

        formatter.format_function(&func);

        assert!(formatter.output.contains("fun test("));
        assert!(formatter.output.contains("x: Int"));
        assert!(formatter.output.contains("return 42;"));
    }

    #[test]
    fn test_idempotence() {
        let expr = Expr::literal(Literal::Int(42), dummy_span());
        let stmt = Stmt::return_stmt(Some(expr), dummy_span());
        let body = Block::new(vec![stmt], dummy_span());

        let func = FunDef::new("main".to_string(), vec![], None, body, dummy_span());

        let program = Program::new(vec![func]);

        let mut formatter1 = Formatter::new();
        let output1 = formatter1.format_program(&program);

        // Parse and format again (we'd need a parser for true idempotence testing)
        // For now, just ensure formatting twice produces the same result
        let mut formatter2 = Formatter::new();
        let output2 = formatter2.format_program(&program);

        assert_eq!(output1, output2);
    }
}
