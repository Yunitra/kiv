//! Identifier and keyword lexing.

use super::cursor::LexerCursor;
use crate::token::{keyword_or_ident, TokenKind};

/// Lexes an identifier or keyword
pub fn lex_identifier(cursor: &mut LexerCursor) -> TokenKind {
    let start = cursor.pos();

    while let Some(c) = cursor.current_char() {
        if c.is_alphanumeric() || c == '_' {
            cursor.advance();
        } else {
            break;
        }
    }

    let text = cursor.slice_from(start);
    keyword_or_ident(text)
}
