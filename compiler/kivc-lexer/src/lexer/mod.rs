//! Main lexer module that orchestrates tokenization.

mod cursor;
mod ident;
mod number;
mod operator;
mod skip;
mod string;

use crate::token::Token;
use crate::token::TokenKind;
use kivc_diagnostics::{DiagnosticsCollector, KivError};
use kivc_span::{SourceFile, Span, TextSize};
use std::sync::Arc;

pub use cursor::LexerCursor;
pub use ident::lex_identifier;
pub use number::lex_number;
pub use operator::lex_operator;
pub use skip::{skip_comments, skip_whitespace};
pub use string::lex_string_literal;

/// A hand-written lexical analyzer that tokenizes Kiv source code.
pub struct Lexer {
    file: Arc<SourceFile>,
    source: String,
    current_pos: usize,
    diagnostics: DiagnosticsCollector,
}

impl Lexer {
    /// Creates a new lexer for the given source file
    pub fn new(file: Arc<SourceFile>) -> Self {
        let source = file.source().to_string();

        Self {
            file,
            source,
            current_pos: 0,
            diagnostics: DiagnosticsCollector::new(),
        }
    }

    /// Returns the collected diagnostics
    pub fn diagnostics(&self) -> &DiagnosticsCollector {
        &self.diagnostics
    }

    /// Consumes the lexer and returns the diagnostics
    pub fn into_diagnostics(self) -> DiagnosticsCollector {
        self.diagnostics
    }

    /// Returns the next token
    pub fn next_token(&mut self) -> Token {
        // Create a cursor for character-level operations
        let mut cursor = LexerCursor::new(&self.source, self.current_pos);

        // Skip whitespace and comments
        skip_whitespace(&mut cursor);
        skip_comments(&mut cursor);

        let start = cursor.pos();

        // Determine token kind and collect any error messages
        let mut error_msg: Option<String> = None;

        let kind = match cursor.current_char() {
            None => TokenKind::Eof,

            // Single-character delimiters
            Some('(') => {
                cursor.advance();
                TokenKind::LParen
            }
            Some(')') => {
                cursor.advance();
                TokenKind::RParen
            }
            Some('{') => {
                cursor.advance();
                TokenKind::LBrace
            }
            Some('}') => {
                cursor.advance();
                TokenKind::RBrace
            }
            Some(':') => {
                cursor.advance();
                TokenKind::Colon
            }
            Some(',') => {
                cursor.advance();
                TokenKind::Comma
            }
            Some(';') => {
                cursor.advance();
                TokenKind::Semicolon
            }

            // Operators (may be multi-character)
            Some(c @ ('+' | '-' | '*' | '/' | '=' | '!' | '<' | '>')) => {
                lex_operator(&mut cursor, c)
            }

            // String literals
            Some('"') => match lex_string_literal(&mut cursor) {
                Ok(s) => TokenKind::TextLit(s),
                Err(msg) => {
                    error_msg = Some(msg);
                    TokenKind::Error
                }
            },

            // Numbers
            Some(c) if c.is_ascii_digit() => match lex_number(&mut cursor) {
                Ok(kind) => kind,
                Err(msg) => {
                    error_msg = Some(msg);
                    TokenKind::Error
                }
            },

            // Identifiers and keywords
            Some(c) if c.is_alphabetic() || c == '_' => lex_identifier(&mut cursor),

            // Unexpected character
            Some(c) => {
                cursor.advance();
                error_msg = Some(format!("unexpected character '{}'", c));
                TokenKind::Error
            }
        };

        // Update lexer position
        self.current_pos = cursor.pos();

        // Report any errors that occurred
        if let Some(msg) = error_msg {
            self.report_error(start, &msg);
        }

        // Create span
        let len = self.current_pos.saturating_sub(start);
        let span = Span::new(
            Arc::clone(&self.file),
            TextSize::from(start as u32),
            TextSize::from(len as u32),
        );

        Token::new(kind, span)
    }

    /// Reports a lexical error
    pub(crate) fn report_error(&mut self, start: usize, message: &str) {
        let len = if self.current_pos > start {
            self.current_pos - start
        } else {
            1
        };
        let span = Span::new(
            Arc::clone(&self.file),
            TextSize::from(start as u32),
            TextSize::from(len as u32),
        );
        let error = KivError::syntax(
            &span,
            message,
            "here",
            None,
            kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
        );
        self.diagnostics.add(error);
    }
}

/// Iterator implementation for Lexer
impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::Eof {
            None
        } else {
            Some(token)
        }
    }
}
