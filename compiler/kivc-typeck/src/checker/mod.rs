//! Type checker modules.

mod context;
mod expr;
mod inference;
mod stmt;

pub use context::TypeChecker;

use kivc_diagnostics::DiagnosticsCollector;
use kivc_hir::HirProgram;

/// Type checks a HIR program
pub fn typecheck_program(program: HirProgram) -> Result<HirProgram, DiagnosticsCollector> {
    let mut checker = TypeChecker::new();
    let checked_program = checker.check_program(program);

    if checker.diagnostics.has_errors() {
        Err(checker.diagnostics)
    } else {
        Ok(checked_program)
    }
}
