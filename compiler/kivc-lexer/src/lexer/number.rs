//! Number literal lexing (integers and floats).

use super::cursor::LexerCursor;
use crate::token::TokenKind;

/// Lexes a number literal (integer or float)
pub fn lex_number(cursor: &mut LexerCursor) -> Result<TokenKind, String> {
    let start = cursor.pos();
    let mut has_dot = false;

    while let Some(c) = cursor.current_char() {
        if c.is_ascii_digit() {
            cursor.advance();
        } else if c == '.' && !has_dot && cursor.peek().is_some_and(|p| p.is_ascii_digit()) {
            has_dot = true;
            cursor.advance();
        } else {
            break;
        }
    }

    let text = cursor.slice_from(start);

    if has_dot {
        text.parse::<f64>()
            .map(TokenKind::FloatLit)
            .map_err(|_| "invalid float literal".to_string())
    } else {
        text.parse::<i64>()
            .map(TokenKind::IntLit)
            .map_err(|_| "invalid integer literal".to_string())
    }
}
