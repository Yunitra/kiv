//! Character-level cursor for navigating source code.

/// A cursor for traversing characters in source code.
pub struct LexerCursor<'a> {
    source: &'a str,
    current_pos: usize,
}

impl<'a> LexerCursor<'a> {
    /// Creates a new cursor at the given position
    pub fn new(source: &'a str, pos: usize) -> Self {
        Self {
            source,
            current_pos: pos,
        }
    }

    /// Returns the current character without consuming it
    pub fn current_char(&self) -> Option<char> {
        self.source[self.current_pos..].chars().next()
    }

    /// Advances to the next character
    pub fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            self.current_pos += ch.len_utf8();
        }
    }

    /// Peeks the next character without consuming
    pub fn peek(&self) -> Option<char> {
        self.source[self.current_pos..].chars().nth(1)
    }

    /// Returns the current byte position
    pub fn pos(&self) -> usize {
        self.current_pos
    }

    /// Returns a slice of the source from start to current position
    pub fn slice_from(&self, start: usize) -> &'a str {
        &self.source[start..self.current_pos]
    }
}
