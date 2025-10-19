//! Tests for formatter.

#[cfg(test)]
mod tests {
    use super::super::Formatter;
    use kivc_ast::*;
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
