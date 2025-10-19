//! Compiler configuration.

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptLevel {
    /// No optimization (debug builds)
    #[default]
    None,
    /// Basic optimizations
    Basic,
    /// Aggressive optimizations
    Aggressive,
}

/// Compiler configuration
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Optimization level
    pub opt_level: OptLevel,

    /// Emit debug information
    pub debug_info: bool,

    /// Stop after a certain stage (for debugging)
    pub stop_after: Option<StopAfter>,
}

/// Stage to stop compilation after
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopAfter {
    Lex,
    Parse,
    Hir,
    TypeCheck,
    Mir,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            opt_level: OptLevel::None,
            debug_info: true,
            stop_after: None,
        }
    }
}

impl CompilerConfig {
    /// Creates a new compiler configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the optimization level
    pub fn with_opt_level(mut self, level: OptLevel) -> Self {
        self.opt_level = level;
        self
    }

    /// Enables or disables debug information
    pub fn with_debug_info(mut self, enabled: bool) -> Self {
        self.debug_info = enabled;
        self
    }

    /// Sets the stage to stop after
    pub fn with_stop_after(mut self, stage: StopAfter) -> Self {
        self.stop_after = Some(stage);
        self
    }
}
