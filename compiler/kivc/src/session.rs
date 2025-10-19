//! Compiler session management.

use crate::config::CompilerConfig;
use kivc_diagnostics::DiagnosticsCollector;

/// A compilation session
#[derive(Debug)]
pub struct CompilerSession {
    /// Compiler configuration
    pub config: CompilerConfig,

    /// Accumulated diagnostics
    pub diagnostics: DiagnosticsCollector,
}

impl CompilerSession {
    /// Creates a new compiler session
    pub fn new(config: CompilerConfig) -> Self {
        Self {
            config,
            diagnostics: DiagnosticsCollector::new(),
        }
    }

    /// Checks if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Gets the diagnostics
    pub fn diagnostics(&self) -> &DiagnosticsCollector {
        &self.diagnostics
    }

    /// Adds diagnostics from another source
    pub fn extend_diagnostics(&mut self, other: DiagnosticsCollector) {
        for error in other.errors() {
            self.diagnostics.add_ref(error);
        }
    }

    /// Clones the diagnostics collector (creates a new one with the same errors)
    pub fn clone_diagnostics(&self) -> DiagnosticsCollector {
        let mut diag = DiagnosticsCollector::new();
        for error in self.diagnostics.errors() {
            diag.add_ref(error);
        }
        diag
    }
}

impl Default for CompilerSession {
    fn default() -> Self {
        Self::new(CompilerConfig::default())
    }
}
