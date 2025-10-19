//! Tests for AST to HIR lowering.

use kivc_hir::lower_program;
use kivc_parser::Parser;
use kivc_span::SourceFile;

fn parse_and_lower(source: &str) -> kivc_hir::HirProgram {
    let file = SourceFile::new("test.kiv".to_string(), source.to_string());
    let mut parser = Parser::new(file);
    let program = parser.parse_program();

    lower_program(program).expect("lowering should succeed")
}

#[test]
fn test_lower_simple_function() {
    let source = r#"
        fun main() {
            let x: Int = 42;
        }
    "#;

    let hir = parse_and_lower(source);
    assert_eq!(hir.functions.len(), 1);
    assert_eq!(hir.functions[0].name, "main");
    assert_eq!(hir.functions[0].body.stmts.len(), 1);
}

#[test]
fn test_lower_variable_resolution() {
    let source = r#"
        fun test() {
            let x: Int = 10;
            let y: Int = x;
        }
    "#;

    let hir = parse_and_lower(source);
    let func = &hir.functions[0];

    // Check that both statements are Let
    assert!(matches!(
        func.body.stmts[0].kind,
        kivc_hir::HirStmtKind::Let { .. }
    ));
    assert!(matches!(
        func.body.stmts[1].kind,
        kivc_hir::HirStmtKind::Let { .. }
    ));
}

#[test]
fn test_lower_function_params() {
    let source = r#"
        fun add(a: Int, b: Int) : Int {
            return a + b;
        }
    "#;

    let hir = parse_and_lower(source);
    let func = &hir.functions[0];

    assert_eq!(func.params.len(), 2);
    assert_eq!(func.params[0].name, "a");
    assert_eq!(func.params[1].name, "b");
    assert!(func.return_type.is_some());
}

#[test]
fn test_lower_binary_expr() {
    let source = r#"
        fun test() {
            let result: Int = 1 + 2 * 3;
        }
    "#;

    let hir = parse_and_lower(source);
    assert_eq!(hir.functions[0].body.stmts.len(), 1);
}

#[test]
fn test_lower_if_expr() {
    let source = r#"
        fun test() {
            let x: Int = if true { 1 } else { 2 };
        }
    "#;

    let hir = parse_and_lower(source);
    assert_eq!(hir.functions[0].body.stmts.len(), 1);
}

#[test]
fn test_lower_assignment() {
    let source = r#"
        fun test() {
            let mut x: Int = 10;
            x = 20;
        }
    "#;

    let hir = parse_and_lower(source);
    assert_eq!(hir.functions[0].body.stmts.len(), 2);
}

#[test]
fn test_lower_function_call() {
    let source = r#"
        fun main() {
            let result: Int = add(1, 2);
        }
    "#;

    let hir = parse_and_lower(source);
    assert_eq!(hir.functions[0].body.stmts.len(), 1);
}

#[test]
fn test_lower_scope_shadowing() {
    let source = r#"
        fun test() {
            let x: Int = 1;
            {
                let x: Int = 2;
            }
        }
    "#;

    // This should succeed - shadowing is allowed
    let hir = parse_and_lower(source);
    assert_eq!(hir.functions.len(), 1);
}

#[test]
fn test_lower_multiple_functions() {
    let source = r#"
        fun foo() {
            let x: Int = 1;
        }
        
        fun bar() {
            let y: Int = 2;
        }
    "#;

    let hir = parse_and_lower(source);
    assert_eq!(hir.functions.len(), 2);
    assert_eq!(hir.functions[0].name, "foo");
    assert_eq!(hir.functions[1].name, "bar");
}
