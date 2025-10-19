//! Tests for HIR to MIR lowering.

use kivc_hir::lower_program;
use kivc_mir::lower_to_mir;
use kivc_parser::Parser;
use kivc_span::SourceFile;
use kivc_typeck::typecheck_program;

fn parse_lower_typecheck_and_mir(source: &str) -> kivc_mir::MirProgram {
    let file = SourceFile::new("test.kiv".to_string(), source.to_string());
    let mut parser = Parser::new(file);
    let program = parser.parse_program();

    let hir = lower_program(program).expect("lowering should succeed");
    let checked_hir = typecheck_program(hir).expect("typechecking should succeed");

    lower_to_mir(checked_hir)
}

#[test]
fn test_mir_simple_function() {
    let source = r#"
        fun main() {
            let x: Int = 42;
        }
    "#;

    let mir = parse_lower_typecheck_and_mir(source);
    assert_eq!(mir.functions.len(), 1);
    assert_eq!(mir.functions[0].name, "main");
    assert!(!mir.functions[0].blocks.is_empty());
}

#[test]
fn test_mir_binary_op() {
    let source = r#"
        fun test() {
            let x: Int = 1 + 2;
        }
    "#;

    let mir = parse_lower_typecheck_and_mir(source);
    assert_eq!(mir.functions.len(), 1);

    let func = &mir.functions[0];
    assert!(!func.blocks.is_empty());

    // Check that there's a BinOp instruction
    let has_binop = func.blocks.iter().any(|block| {
        block
            .instructions
            .iter()
            .any(|instr| matches!(instr, kivc_mir::MirInstr::BinOp { .. }))
    });
    assert!(has_binop);
}

#[test]
fn test_mir_return() {
    let source = r#"
        fun get_value() : Int {
            return 100;
        }
    "#;

    let mir = parse_lower_typecheck_and_mir(source);
    let func = &mir.functions[0];

    // Check that there's a Return terminator
    let has_return = func
        .blocks
        .iter()
        .any(|block| matches!(block.terminator, kivc_mir::MirTerminator::Return { .. }));
    assert!(has_return);
}

#[test]
fn test_mir_function_params() {
    let source = r#"
        fun add(a: Int, b: Int) : Int {
            return a + b;
        }
    "#;

    let mir = parse_lower_typecheck_and_mir(source);
    let func = &mir.functions[0];

    assert_eq!(func.params.len(), 2);
}

#[test]
fn test_mir_assignment() {
    let source = r#"
        fun test() {
            let mut x: Int = 10;
            x = 20;
        }
    "#;

    let mir = parse_lower_typecheck_and_mir(source);
    let func = &mir.functions[0];

    // Check for Assign instructions
    let assign_count = func
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .filter(|instr| matches!(instr, kivc_mir::MirInstr::Assign { .. }))
        .count();

    assert!(assign_count >= 2); // At least two assignments
}
