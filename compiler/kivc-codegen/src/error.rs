//! Error types for code generation.

use kivc_diagnostics::KivError;
use kivc_span::{SourceFile, Span};
use text_size::TextSize;
use thiserror::Error;

pub type CodegenResult<T> = Result<T, CodegenError>;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("LLVM error: {0}")]
    LlvmError(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Codegen error: {0}")]
    General(String),
}

impl CodegenError {
    fn dummy_span() -> Span {
        let file = SourceFile::new("<codegen>".to_string(), String::new());
        Span::new(file, TextSize::from(0), TextSize::from(0))
    }
}

impl From<CodegenError> for KivError {
    fn from(err: CodegenError) -> Self {
        let span = CodegenError::dummy_span();
        KivError::syntax(
            &span,
            err.to_string(),
            "codegen error",
            None,
            kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
        )
    }
}
