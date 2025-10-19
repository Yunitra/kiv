//! Lint rules for Kiv code.

mod const_mut;
mod redundant_else;
mod unreachable;
mod unused_var;

pub use const_mut::ConstMutable;
pub use redundant_else::RedundantElse;
pub use unreachable::UnreachableCode;
pub use unused_var::UnusedVariable;

use kivc_diagnostics::DiagnosticsCollector;
use kivc_hir::HirProgram;

/// A lint rule
pub trait LintRule {
    /// The name of the lint rule
    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    /// Runs the lint rule on a HIR program
    fn check(&self, program: &HirProgram, diagnostics: &mut DiagnosticsCollector);
}
