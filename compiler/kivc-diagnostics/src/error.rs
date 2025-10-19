use crate::error_code::ErrorCode;
use kivc_span::Span;
use miette::{Diagnostic, SourceSpan};
use std::fmt;
use std::sync::Arc;
use thiserror::Error;

/// Main error type for the Kiv compiler.
///
/// Designed to work seamlessly with miette for beautiful error reporting.
#[derive(Clone, Error, Diagnostic)]
pub enum KivError {
    /// Syntax error with source location
    #[error("{message}")]
    Syntax {
        #[source_code]
        src: Arc<dyn miette::SourceCode + Send + Sync>,
        #[label("{label}")]
        span: SourceSpan,
        message: String,
        label: String,
        #[help]
        help: Option<String>,
        code: ErrorCode,
    },

    /// Type error with source location
    #[error("{message}")]
    Type {
        #[source_code]
        src: Arc<dyn miette::SourceCode + Send + Sync>,
        #[label("{label}")]
        span: SourceSpan,
        message: String,
        label: String,
        #[help]
        help: Option<String>,
        code: ErrorCode,
    },

    /// IO error without source location
    #[error("{message}")]
    Io {
        message: String,
        #[help]
        help: Option<String>,
        code: ErrorCode,
    },

    /// Generic error without source location
    #[error("{message}")]
    Generic {
        message: String,
        #[help]
        help: Option<String>,
    },
}

impl KivError {
    /// Creates a syntax error
    pub fn syntax(
        span: &Span,
        message: impl Into<String>,
        label: impl Into<String>,
        help: Option<String>,
        code: ErrorCode,
    ) -> Self {
        let file = span.file();
        let source = miette::NamedSource::new(file.name(), file.source().to_string());

        Self::Syntax {
            src: Arc::new(source),
            span: SourceSpan::new(
                (u32::from(span.offset()) as usize).into(),
                u32::from(span.len()) as usize,
            ),
            message: message.into(),
            label: label.into(),
            help,
            code,
        }
    }

    /// Creates a type error
    pub fn type_error(
        span: &Span,
        message: impl Into<String>,
        label: impl Into<String>,
        help: Option<String>,
        code: ErrorCode,
    ) -> Self {
        let file = span.file();
        let source = miette::NamedSource::new(file.name(), file.source().to_string());

        Self::Type {
            src: Arc::new(source),
            span: SourceSpan::new(
                (u32::from(span.offset()) as usize).into(),
                u32::from(span.len()) as usize,
            ),
            message: message.into(),
            label: label.into(),
            help,
            code,
        }
    }

    /// Creates an IO error
    pub fn io(message: impl Into<String>, help: Option<String>, code: ErrorCode) -> Self {
        Self::Io {
            message: message.into(),
            help,
            code,
        }
    }

    /// Creates a generic error
    pub fn generic(message: impl Into<String>, help: Option<String>) -> Self {
        Self::Generic {
            message: message.into(),
            help,
        }
    }

    /// Creates a warning (represented as a syntax error with a special code)
    pub fn warning(span: Span, message: impl Into<String>, help: impl Into<String>) -> Self {
        Self::syntax(
            &span,
            message,
            "warning",
            Some(help.into()),
            ErrorCode::new("lint", 0),
        )
    }
}

// Manual Debug implementation to handle the non-Debug SourceCode trait object
impl fmt::Debug for KivError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax {
                message,
                label,
                help,
                code,
                ..
            } => f
                .debug_struct("Syntax")
                .field("message", message)
                .field("label", label)
                .field("help", help)
                .field("code", code)
                .finish_non_exhaustive(),
            Self::Type {
                message,
                label,
                help,
                code,
                ..
            } => f
                .debug_struct("Type")
                .field("message", message)
                .field("label", label)
                .field("help", help)
                .field("code", code)
                .finish_non_exhaustive(),
            Self::Io {
                message,
                help,
                code,
            } => f
                .debug_struct("Io")
                .field("message", message)
                .field("help", help)
                .field("code", code)
                .finish(),
            Self::Generic { message, help } => f
                .debug_struct("Generic")
                .field("message", message)
                .field("help", help)
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_code::*;
    use kivc_span::SourceFile;
    use miette::Report;

    #[test]
    fn test_syntax_error() {
        let file = SourceFile::new("test.kiv".to_string(), "let x = 42;".to_string());
        let span = Span::new(Arc::clone(&file), 4.into(), 1.into());

        let error = KivError::syntax(
            &span,
            "unexpected token",
            "expected identifier",
            Some("try using a valid identifier name".to_string()),
            E001_UNEXPECTED_TOKEN,
        );

        assert!(matches!(error, KivError::Syntax { .. }));
    }

    #[test]
    fn test_type_error() {
        let file = SourceFile::new("test.kiv".to_string(), "let x: Int = true;".to_string());
        let span = Span::new(Arc::clone(&file), 13.into(), 4.into());

        let error = KivError::type_error(
            &span,
            "type mismatch",
            "expected Int, found Bool",
            Some("consider changing the type annotation".to_string()),
            E100_TYPE_MISMATCH,
        );

        assert!(matches!(error, KivError::Type { .. }));
    }

    #[test]
    fn test_io_error() {
        let error = KivError::io(
            "file not found",
            Some("check that the file exists".to_string()),
            E200_FILE_NOT_FOUND,
        );

        assert!(matches!(error, KivError::Io { .. }));
    }

    #[test]
    fn test_generic_error() {
        let error = KivError::generic("something went wrong", None);
        assert!(matches!(error, KivError::Generic { .. }));
    }

    #[test]
    fn test_error_can_be_converted_to_report() {
        let file = SourceFile::new("test.kiv".to_string(), "let x = 42;".to_string());
        let span = Span::new(Arc::clone(&file), 4.into(), 1.into());

        let error = KivError::syntax(
            &span,
            "unexpected token",
            "here",
            None,
            E001_UNEXPECTED_TOKEN,
        );

        let _report: Report = error.into();
    }
}
