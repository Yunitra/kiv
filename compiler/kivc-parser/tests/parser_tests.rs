//! Integration tests for the parser.

use kivc_parser::Parser;
use kivc_span::SourceFile;

fn parse_source(source: &str) -> kivc_ast::Program {
    let file = SourceFile::new("test.kiv".to_string(), source.to_string());
    let mut parser = Parser::new(file);
    parser.parse_program()
}

#[test]
fn test_parse_simple_function() {
    let source = r#"
        fun main() {
            let x: Int = 42;
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "main");
    assert_eq!(program.functions[0].params.len(), 0);
    assert_eq!(program.functions[0].body.stmts.len(), 1);
}

#[test]
fn test_parse_function_with_params() {
    let source = r#"
        fun add(a: Int, b: Int) : Int {
            return a + b;
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].params.len(), 2);
    assert!(program.functions[0].ret_ty.is_some());
}

#[test]
fn test_parse_binary_operators() {
    let source = r#"
        fun test() {
            let a: Int = 1 + 2 * 3;
            let b: Bool = 5 > 3;
            let c: Bool = 10 == 10;
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions[0].body.stmts.len(), 3);
}

#[test]
fn test_parse_if_expression() {
    let source = r#"
        fun test() {
            let x: Int = if true { 1 } else { 2 };
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions[0].body.stmts.len(), 1);
}

#[test]
fn test_parse_function_call() {
    let source = r#"
        fun main() {
            let result: Int = add(1, 2);
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions[0].body.stmts.len(), 1);
}

#[test]
fn test_parse_assignment() {
    let source = r#"
        fun main() {
            let mut x: Int = 10;
            x = 20;
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions[0].body.stmts.len(), 2);
}

#[test]
fn test_parse_const() {
    let source = r#"
        fun main() {
            const PI = 3.14;
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions[0].body.stmts.len(), 1);
}

#[test]
fn test_parse_return() {
    let source = r#"
        fun get_value() : Int {
            return 42;
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions[0].body.stmts.len(), 1);
}

#[test]
fn test_parse_multiple_functions() {
    let source = r#"
        fun foo() {
            let x: Int = 1;
        }
        
        fun bar() {
            let y: Int = 2;
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions.len(), 2);
    assert_eq!(program.functions[0].name, "foo");
    assert_eq!(program.functions[1].name, "bar");
}

#[test]
fn test_parse_operator_precedence() {
    let source = r#"
        fun test() {
            let x: Int = 1 + 2 * 3;
            let y: Int = (1 + 2) * 3;
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions[0].body.stmts.len(), 2);
}

#[test]
fn test_parse_string_literals() {
    let source = r#"
        fun main() {
            let msg: Text = "Hello, world!";
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions[0].body.stmts.len(), 1);
}

#[test]
fn test_parse_bool_literals() {
    let source = r#"
        fun main() {
            let t: Bool = true;
            let f: Bool = false;
        }
    "#;

    let program = parse_source(source);
    assert_eq!(program.functions[0].body.stmts.len(), 2);
}
