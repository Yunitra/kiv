use std::sync::Arc;
use text_size::TextSize;

/// Represents a single source file with its contents and line information.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// The name or path of the source file
    name: String,
    /// The complete source code content
    source: String,
    /// Byte offsets where each line starts (for line/column conversion)
    line_starts: Vec<TextSize>,
}

impl SourceFile {
    /// Creates a new SourceFile from the given name and source code.
    ///
    /// Automatically computes line start positions for efficient line/column lookups.
    pub fn new(name: String, source: String) -> Arc<Self> {
        let mut line_starts = vec![TextSize::from(0)];

        for (idx, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(TextSize::from((idx + 1) as u32));
            }
        }

        Arc::new(Self {
            name,
            source,
            line_starts,
        })
    }

    /// Returns the file name
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the complete source code
    #[inline]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns a slice of the source code at the given byte range
    pub fn source_slice(&self, start: TextSize, end: TextSize) -> &str {
        let start = u32::from(start) as usize;
        let end = u32::from(end) as usize;
        &self.source[start..end]
    }

    /// Converts a byte offset to a (line, column) pair (both 0-based)
    pub fn line_col(&self, offset: TextSize) -> (usize, usize) {
        let line = self
            .line_starts
            .binary_search(&offset)
            .unwrap_or_else(|next_line| next_line - 1);

        let line_start = self.line_starts[line];
        let col = u32::from(offset - line_start) as usize;

        (line, col)
    }

    /// Returns the byte offset for the given (line, column) pair (both 0-based)
    pub fn offset(&self, line: usize, col: usize) -> Option<TextSize> {
        let line_start = *self.line_starts.get(line)?;
        let offset = line_start + TextSize::from(col as u32);

        // Validate that offset is within bounds
        if u32::from(offset) as usize <= self.source.len() {
            Some(offset)
        } else {
            None
        }
    }

    /// Returns the number of lines in the source file
    #[inline]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file() {
        let file = SourceFile::new("test.kiv".to_string(), String::new());
        assert_eq!(file.name(), "test.kiv");
        assert_eq!(file.source(), "");
        assert_eq!(file.line_count(), 1);
    }

    #[test]
    fn test_single_line() {
        let file = SourceFile::new("test.kiv".to_string(), "hello world".to_string());
        assert_eq!(file.line_count(), 1);
        assert_eq!(file.line_col(TextSize::from(0)), (0, 0));
        assert_eq!(file.line_col(TextSize::from(6)), (0, 6));
    }

    #[test]
    fn test_multiple_lines() {
        let source = "line1\nline2\nline3".to_string();
        let file = SourceFile::new("test.kiv".to_string(), source);

        assert_eq!(file.line_count(), 3);
        assert_eq!(file.line_col(TextSize::from(0)), (0, 0)); // 'l' in line1
        assert_eq!(file.line_col(TextSize::from(6)), (1, 0)); // 'l' in line2
        assert_eq!(file.line_col(TextSize::from(12)), (2, 0)); // 'l' in line3
    }

    #[test]
    fn test_line_col_conversion() {
        let source = "hello\nworld\n".to_string();
        let file = SourceFile::new("test.kiv".to_string(), source);

        // Test round-trip conversion
        let offset = file.offset(1, 3).unwrap();
        assert_eq!(file.line_col(offset), (1, 3));
    }

    #[test]
    fn test_source_slice() {
        let file = SourceFile::new("test.kiv".to_string(), "hello world".to_string());
        let slice = file.source_slice(TextSize::from(0), TextSize::from(5));
        assert_eq!(slice, "hello");
    }

    #[test]
    fn test_offset_out_of_bounds() {
        let file = SourceFile::new("test.kiv".to_string(), "hello".to_string());
        assert!(file.offset(0, 100).is_none());
        assert!(file.offset(10, 0).is_none());
    }
}
