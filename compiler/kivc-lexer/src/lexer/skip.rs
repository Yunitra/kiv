//! Functions for skipping whitespace and comments.

use super::cursor::LexerCursor;

/// Skips whitespace characters
pub fn skip_whitespace(cursor: &mut LexerCursor) {
    while let Some(c) = cursor.current_char() {
        if c.is_whitespace() {
            cursor.advance();
        } else {
            break;
        }
    }
}

/// Skips single-line comments (// to end of line)
pub fn skip_comments(cursor: &mut LexerCursor) {
    while cursor.current_char() == Some('/') && cursor.peek() == Some('/') {
        // Skip the //
        cursor.advance();
        cursor.advance();

        // Skip until end of line
        while let Some(c) = cursor.current_char() {
            if c == '\n' {
                break;
            }
            cursor.advance();
        }

        skip_whitespace(cursor);
    }
}
