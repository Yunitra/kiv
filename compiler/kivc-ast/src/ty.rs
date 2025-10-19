use kivc_span::Span;

/// A type annotation in the AST
#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

/// The kind of type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    /// Integer type
    Int,
    /// Floating-point type
    Float,
    /// Boolean type
    Bool,
    /// Text (string) type
    Text,
    /// Unit type (no value)
    Unit,
}

impl Type {
    /// Creates a new type with the given kind and span
    pub fn new(kind: TypeKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivc_span::SourceFile;
    use std::sync::Arc;

    #[test]
    fn test_type_creation() {
        let file = SourceFile::new("test.kiv".to_string(), "Int".to_string());
        let span = Span::new(Arc::clone(&file), 0.into(), 3.into());
        let ty = Type::new(TypeKind::Int, span);

        assert!(matches!(ty.kind, TypeKind::Int));
    }
}
