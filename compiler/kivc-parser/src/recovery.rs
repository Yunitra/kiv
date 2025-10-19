//! Error recovery strategies for the parser.

use crate::parser::Parser;
use kivc_lexer::TokenKind;

/// Synchronizes the parser to a known recovery point.
/// Advances until we hit a statement or declaration boundary.
pub fn synchronize(parser: &mut Parser) {
    while !parser.is_at_end() {
        // After a semicolon, we're likely at a statement boundary
        if parser.current().kind == TokenKind::Semicolon {
            parser.advance();
            return;
        }

        // These keywords start new declarations
        match parser.current().kind {
            TokenKind::Fun
            | TokenKind::Let
            | TokenKind::Const
            | TokenKind::If
            | TokenKind::Return => return,
            _ => {
                parser.advance();
            }
        }
    }
}
