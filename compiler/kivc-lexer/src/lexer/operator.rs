//! Operator lexing (single and multi-character operators).

use super::cursor::LexerCursor;
use crate::token::TokenKind;

/// Lexes an operator (single or multi-character)
pub fn lex_operator(cursor: &mut LexerCursor, first: char) -> TokenKind {
    cursor.advance(); // consume first character

    match first {
        '+' => TokenKind::Plus,
        '-' => TokenKind::Minus,
        '*' => TokenKind::Star,
        '/' => TokenKind::Slash,
        '=' => {
            if cursor.current_char() == Some('=') {
                cursor.advance();
                TokenKind::EqEq
            } else {
                TokenKind::Eq
            }
        }
        '!' => {
            if cursor.current_char() == Some('=') {
                cursor.advance();
                TokenKind::NotEq
            } else {
                // '!' alone is an error, but we return Error and let the caller report it
                TokenKind::Error
            }
        }
        '<' => {
            if cursor.current_char() == Some('=') {
                cursor.advance();
                TokenKind::Le
            } else {
                TokenKind::Lt
            }
        }
        '>' => {
            if cursor.current_char() == Some('=') {
                cursor.advance();
                TokenKind::Ge
            } else {
                TokenKind::Gt
            }
        }
        _ => TokenKind::Error,
    }
}
