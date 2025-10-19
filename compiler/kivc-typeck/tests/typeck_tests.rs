//! Type checking tests.

use kivc_hir::lower_program;
use kivc_parser::Parser;
use kivc_span::SourceFile;
use kivc_typeck::typecheck_program;

fn parse_lower_and_typecheck(source: &str) -> Result<(), String> {
    let file = SourceFile::new("test.kiv".to_string(), source.to_string());
    let mut parser = Parser::new(file);
    let program = parser.parse_program();

    let hir = lower_program(program).map_err(|_| "lowering failed".to_string())?;

    typecheck_program(hir).map_err(|_| "type checking failed".to_string())?;

    Ok(())
}

#[test]
fn test_typecheck_simple() {
    let source = r#"
        fun main() {
            let x: Int = 42;
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_ok());
}

#[test]
fn test_typecheck_binary_op() {
    let source = r#"
        fun main() {
            let x: Int = 1 + 2;
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_ok());
}

#[test]
fn test_typecheck_type_mismatch() {
    let source = r#"
        fun main() {
            let x: Int = true;
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_err());
}

#[test]
fn test_typecheck_immutable_assignment() {
    let source = r#"
        fun main() {
            let x: Int = 10;
            x = 20;
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_err());
}

#[test]
fn test_typecheck_mutable_assignment() {
    let source = r#"
        fun main() {
            let mut x: Int = 10;
            x = 20;
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_ok());
}

#[test]
fn test_typecheck_if_condition() {
    let source = r#"
        fun main() {
            let x: Int = if true { 1 } else { 2 };
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_ok());
}

#[test]
fn test_typecheck_if_non_bool_condition() {
    let source = r#"
        fun main() {
            let x: Int = if 42 { 1 } else { 2 };
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_err());
}

#[test]
fn test_typecheck_comparison() {
    let source = r#"
        fun main() {
            let x: Bool = 5 > 3;
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_ok());
}

#[test]
fn test_typecheck_return_type() {
    let source = r#"
        fun get_int() : Int {
            return 42;
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_ok());
}

#[test]
fn test_typecheck_return_type_mismatch() {
    let source = r#"
        fun get_int() : Int {
            return true;
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_err());
}

#[test]
fn test_typecheck_type_inference() {
    let source = r#"
        fun main() {
            let x = 42;
            let y = x + 10;
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_ok());
}

#[test]
fn test_typecheck_function_params() {
    let source = r#"
        fun add(a: Int, b: Int) : Int {
            return a + b;
        }
    "#;

    assert!(parse_lower_and_typecheck(source).is_ok());
}
