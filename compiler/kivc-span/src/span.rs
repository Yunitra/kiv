use crate::SourceFile;
use std::sync::Arc;
use text_size::TextSize;

/// A span representing a region in a source file.
///
/// Spans are lightweight and can be freely cloned. They internally use Arc
/// to share the SourceFile reference efficiently.
#[derive(Debug, Clone)]
pub struct Span {
    file: Arc<SourceFile>,
    offset: TextSize,
    len: TextSize,
}

impl Span {
    /// Creates a new span with the given file, offset, and length
    pub fn new(file: Arc<SourceFile>, offset: TextSize, len: TextSize) -> Self {
        Self { file, offset, len }
    }

    /// Returns a reference to the source file
    #[inline]
    pub fn file(&self) -> &Arc<SourceFile> {
        &self.file
    }

    /// Returns the starting byte offset
    #[inline]
    pub fn offset(&self) -> TextSize {
        self.offset
    }

    /// Returns the length in bytes
    #[inline]
    pub fn len(&self) -> TextSize {
        self.len
    }

    /// Returns the ending byte offset (exclusive)
    #[inline]
    pub fn end(&self) -> TextSize {
        self.offset + self.len
    }

    /// Returns true if this span has zero length
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == TextSize::from(0)
    }

    /// Returns the source text covered by this span
    pub fn text(&self) -> &str {
        self.file.source_slice(self.offset, self.end())
    }

    /// Returns the starting (line, column) position (both 0-based)
    pub fn start_line_col(&self) -> (usize, usize) {
        self.file.line_col(self.offset)
    }

    /// Returns the ending (line, column) position (both 0-based)
    pub fn end_line_col(&self) -> (usize, usize) {
        self.file.line_col(self.end())
    }

    /// Checks if this span contains the given offset
    pub fn contains(&self, offset: TextSize) -> bool {
        self.offset <= offset && offset < self.end()
    }

    /// Checks if this span contains another span
    pub fn contains_span(&self, other: &Span) -> bool {
        Arc::ptr_eq(&self.file, &other.file)
            && self.offset <= other.offset
            && other.end() <= self.end()
    }

    /// Merges this span with another, returning a span that covers both.
    ///
    /// Returns None if the spans are from different files.
    pub fn merge(&self, other: &Span) -> Option<Span> {
        if !Arc::ptr_eq(&self.file, &other.file) {
            return None;
        }

        let start = self.offset.min(other.offset);
        let end = self.end().max(other.end());

        Some(Span::new(Arc::clone(&self.file), start, end - start))
    }

    /// Extends this span to include another span.
    ///
    /// Panics if the spans are from different files.
    pub fn extend(&mut self, other: &Span) {
        assert!(
            Arc::ptr_eq(&self.file, &other.file),
            "Cannot extend spans from different files"
        );

        let new_start = self.offset.min(other.offset);
        let new_end = self.end().max(other.end());

        self.offset = new_start;
        self.len = new_end - new_start;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file() -> Arc<SourceFile> {
        SourceFile::new("test.kiv".to_string(), "hello world\nfoo bar".to_string())
    }

    #[test]
    fn test_span_creation() {
        let file = create_test_file();
        let span = Span::new(Arc::clone(&file), TextSize::from(0), TextSize::from(5));

        assert_eq!(span.offset(), TextSize::from(0));
        assert_eq!(span.len(), TextSize::from(5));
        assert_eq!(span.end(), TextSize::from(5));
        assert_eq!(span.text(), "hello");
    }

    #[test]
    fn test_span_contains() {
        let file = create_test_file();
        let span = Span::new(Arc::clone(&file), TextSize::from(0), TextSize::from(5));

        assert!(span.contains(TextSize::from(0)));
        assert!(span.contains(TextSize::from(4)));
        assert!(!span.contains(TextSize::from(5)));
        assert!(!span.contains(TextSize::from(10)));
    }

    #[test]
    fn test_span_contains_span() {
        let file = create_test_file();
        let outer = Span::new(Arc::clone(&file), TextSize::from(0), TextSize::from(10));
        let inner = Span::new(Arc::clone(&file), TextSize::from(2), TextSize::from(5));

        assert!(outer.contains_span(&inner));
        assert!(!inner.contains_span(&outer));
    }

    #[test]
    fn test_span_merge() {
        let file = create_test_file();
        let span1 = Span::new(Arc::clone(&file), TextSize::from(0), TextSize::from(5));
        let span2 = Span::new(Arc::clone(&file), TextSize::from(6), TextSize::from(5));

        let merged = span1.merge(&span2).unwrap();
        assert_eq!(merged.offset(), TextSize::from(0));
        assert_eq!(merged.len(), TextSize::from(11));
        assert_eq!(merged.text(), "hello world");
    }

    #[test]
    fn test_span_merge_different_files() {
        let file1 = create_test_file();
        let file2 = SourceFile::new("other.kiv".to_string(), "other".to_string());

        let span1 = Span::new(Arc::clone(&file1), TextSize::from(0), TextSize::from(5));
        let span2 = Span::new(Arc::clone(&file2), TextSize::from(0), TextSize::from(5));

        assert!(span1.merge(&span2).is_none());
    }

    #[test]
    fn test_span_extend() {
        let file = create_test_file();
        let mut span1 = Span::new(Arc::clone(&file), TextSize::from(0), TextSize::from(5));
        let span2 = Span::new(Arc::clone(&file), TextSize::from(6), TextSize::from(5));

        span1.extend(&span2);
        assert_eq!(span1.offset(), TextSize::from(0));
        assert_eq!(span1.len(), TextSize::from(11));
    }

    #[test]
    fn test_empty_span() {
        let file = create_test_file();
        let span = Span::new(Arc::clone(&file), TextSize::from(5), TextSize::from(0));

        assert!(span.is_empty());
        assert_eq!(span.text(), "");
    }

    #[test]
    fn test_line_col() {
        let file = create_test_file();
        let span = Span::new(Arc::clone(&file), TextSize::from(0), TextSize::from(5));

        assert_eq!(span.start_line_col(), (0, 0));
        assert_eq!(span.end_line_col(), (0, 5));
    }

    #[test]
    #[should_panic(expected = "Cannot extend spans from different files")]
    fn test_extend_different_files_panics() {
        let file1 = create_test_file();
        let file2 = SourceFile::new("other.kiv".to_string(), "other".to_string());

        let mut span1 = Span::new(Arc::clone(&file1), TextSize::from(0), TextSize::from(5));
        let span2 = Span::new(Arc::clone(&file2), TextSize::from(0), TextSize::from(5));

        span1.extend(&span2);
    }
}
