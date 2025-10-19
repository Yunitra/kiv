use crate::KivError;

/// A collector for multiple diagnostics.
///
/// Useful for collecting errors during compilation phases without immediately
/// aborting, allowing reporting of multiple errors at once.
#[derive(Debug, Default)]
pub struct DiagnosticsCollector {
    errors: Vec<KivError>,
}

impl DiagnosticsCollector {
    /// Creates a new empty collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an error to the collection
    pub fn add(&mut self, error: KivError) {
        self.errors.push(error);
    }

    /// Adds an error by reference (cloning it)
    pub fn add_ref(&mut self, error: &KivError) {
        self.errors.push(error.clone());
    }

    /// Clones the collector (creates a new one with the same errors)
    pub fn clone_box(&self) -> DiagnosticsCollector {
        let mut diag = DiagnosticsCollector::new();
        for error in &self.errors {
            diag.add_ref(error);
        }
        diag
    }

    /// Returns true if any errors have been collected
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns the number of errors collected
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Returns a reference to the collected errors
    pub fn errors(&self) -> &[KivError] {
        &self.errors
    }

    /// Consumes the collector and returns the collected errors
    pub fn into_errors(self) -> Vec<KivError> {
        self.errors
    }

    /// Clears all collected errors
    pub fn clear(&mut self) {
        self.errors.clear();
    }

    /// Creates a Result based on whether errors were collected
    ///
    /// Returns Ok(value) if no errors, Err(errors) otherwise
    pub fn finish<T>(self, value: T) -> Result<T, Vec<KivError>> {
        if self.has_errors() {
            Err(self.errors)
        } else {
            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_code::*;
    use kivc_span::{SourceFile, Span};
    use std::sync::Arc;

    #[test]
    fn test_empty_collector() {
        let collector = DiagnosticsCollector::new();
        assert!(!collector.has_errors());
        assert_eq!(collector.error_count(), 0);
    }

    #[test]
    fn test_add_error() {
        let mut collector = DiagnosticsCollector::new();

        let file = SourceFile::new("test.kiv".to_string(), "let x = 42;".to_string());
        let span = Span::new(Arc::clone(&file), 4.into(), 1.into());

        collector.add(KivError::syntax(
            &span,
            "test error",
            "here",
            None,
            E001_UNEXPECTED_TOKEN,
        ));

        assert!(collector.has_errors());
        assert_eq!(collector.error_count(), 1);
    }

    #[test]
    fn test_multiple_errors() {
        let mut collector = DiagnosticsCollector::new();

        let file = SourceFile::new("test.kiv".to_string(), "let x = 42;".to_string());
        let span1 = Span::new(Arc::clone(&file), 0.into(), 3.into());
        let span2 = Span::new(Arc::clone(&file), 4.into(), 1.into());

        collector.add(KivError::syntax(
            &span1,
            "error 1",
            "here",
            None,
            E001_UNEXPECTED_TOKEN,
        ));
        collector.add(KivError::syntax(
            &span2,
            "error 2",
            "here",
            None,
            E002_EXPECTED_TOKEN,
        ));

        assert_eq!(collector.error_count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut collector = DiagnosticsCollector::new();

        let file = SourceFile::new("test.kiv".to_string(), "let x = 42;".to_string());
        let span = Span::new(Arc::clone(&file), 4.into(), 1.into());

        collector.add(KivError::syntax(
            &span,
            "test error",
            "here",
            None,
            E001_UNEXPECTED_TOKEN,
        ));

        collector.clear();
        assert!(!collector.has_errors());
        assert_eq!(collector.error_count(), 0);
    }

    #[test]
    fn test_finish_success() {
        let collector = DiagnosticsCollector::new();
        let result = collector.finish(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_finish_failure() {
        let mut collector = DiagnosticsCollector::new();

        let file = SourceFile::new("test.kiv".to_string(), "let x = 42;".to_string());
        let span = Span::new(Arc::clone(&file), 4.into(), 1.into());

        collector.add(KivError::syntax(
            &span,
            "test error",
            "here",
            None,
            E001_UNEXPECTED_TOKEN,
        ));

        let result = collector.finish(42);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 1);
    }
}
