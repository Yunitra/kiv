//! Main compilation pipeline.

use crate::config::{CompilerConfig, StopAfter};
use crate::session::CompilerSession;
use kivc_ast::Program as AstProgram;
use kivc_codegen::generate_llvm_ir;
use kivc_diagnostics::DiagnosticsCollector;
use kivc_hir::{lower_program, HirProgram};
use kivc_mir::{lower_to_mir, MirProgram};
use kivc_parser::Parser;
use kivc_span::SourceFile;
use kivc_typeck::typecheck_program;
use std::sync::Arc;

/// Result of compilation
#[derive(Debug)]
pub enum CompileResult {
    /// Compilation succeeded, LLVM IR is available
    Success(String),

    /// Compilation stopped at AST stage
    StoppedAtAst(AstProgram),

    /// Compilation stopped at HIR stage
    StoppedAtHir(HirProgram),

    /// Compilation stopped at MIR stage
    StoppedAtMir(MirProgram),

    /// Compilation failed with errors
    Failed(DiagnosticsCollector),
}

/// Compiles source code to MIR
pub fn compile_source(
    filename: String,
    source: String,
    config: CompilerConfig,
) -> (CompilerSession, CompileResult) {
    let mut session = CompilerSession::new(config);

    // Create source file
    let file = SourceFile::new(filename, source);

    // Parse
    let ast = match compile_to_ast(file) {
        Ok(ast) => ast,
        Err(diag) => {
            session.extend_diagnostics(diag);
            let failed_diag = session.diagnostics().clone_box();
            return (session, CompileResult::Failed(failed_diag));
        }
    };

    if session.config.stop_after == Some(StopAfter::Parse) {
        return (session, CompileResult::StoppedAtAst(ast));
    }

    // Lower to HIR
    let hir = match compile_to_hir(ast) {
        Ok(hir) => hir,
        Err(diag) => {
            session.extend_diagnostics(diag);
            let failed_diag = session.diagnostics().clone_box();
            return (session, CompileResult::Failed(failed_diag));
        }
    };

    if session.config.stop_after == Some(StopAfter::Hir) {
        return (session, CompileResult::StoppedAtHir(hir));
    }

    // Type check
    let checked_hir = match typecheck_program(hir) {
        Ok(hir) => hir,
        Err(diag) => {
            session.extend_diagnostics(diag);
            let failed_diag = session.diagnostics().clone_box();
            return (session, CompileResult::Failed(failed_diag));
        }
    };

    if session.config.stop_after == Some(StopAfter::TypeCheck) {
        return (session, CompileResult::StoppedAtHir(checked_hir));
    }

    // Lower to MIR
    let mir = lower_to_mir(checked_hir);

    if session.config.stop_after == Some(StopAfter::Mir) {
        return (session, CompileResult::StoppedAtMir(mir.clone()));
    }

    // Generate LLVM IR
    let llvm_ir = match generate_llvm_ir(&mir) {
        Ok(code) => code,
        Err(err) => {
            // Convert codegen error to KivError
            let kiv_err = kivc_diagnostics::KivError::from(err);
            let mut diag = DiagnosticsCollector::new();
            diag.add(kiv_err);
            session.extend_diagnostics(diag);
            let failed_diag = session.diagnostics().clone_box();
            return (session, CompileResult::Failed(failed_diag));
        }
    };

    (session, CompileResult::Success(llvm_ir))
}

/// Compiles source to AST
pub fn compile_to_ast(file: Arc<SourceFile>) -> Result<AstProgram, DiagnosticsCollector> {
    let mut parser = Parser::new(file);
    let program = parser.parse_program();

    if parser.diagnostics().has_errors() {
        // Clone the diagnostics collector by creating a new one with the same errors
        let mut diag = DiagnosticsCollector::new();
        for error in parser.diagnostics().errors() {
            diag.add_ref(error);
        }
        return Err(diag);
    }

    Ok(program)
}

/// Compiles AST to HIR
pub fn compile_to_hir(ast: AstProgram) -> Result<HirProgram, DiagnosticsCollector> {
    lower_program(ast)
}

/// Compiles HIR to MIR
pub fn compile_to_mir(hir: HirProgram) -> MirProgram {
    lower_to_mir(hir)
}
