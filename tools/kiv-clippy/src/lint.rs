//! Placeholder lint module (to be fully implemented later)

use kivc_diagnostics::DiagnosticsCollector;
use kivc_hir::HirProgram;

pub trait LintRule {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    fn check(&self, _program: &HirProgram, _diagnostics: &mut DiagnosticsCollector) {}
}

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![]
}
