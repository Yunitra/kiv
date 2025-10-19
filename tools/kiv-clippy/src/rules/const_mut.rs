//! Const mutable detection (placeholder).

use super::LintRule;
use kivc_diagnostics::DiagnosticsCollector;
use kivc_hir::HirProgram;

/// Detects non-literal const values (placeholder)
pub struct ConstMutable;

impl LintRule for ConstMutable {
    fn name(&self) -> &'static str {
        "const_mutable"
    }

    fn check(&self, program: &HirProgram, _diagnostics: &mut DiagnosticsCollector) {
        // This would require tracking const declarations in HIR
        // For now, this is a placeholder
        for _func in &program.functions {
            // Check const declarations
        }
    }
}
