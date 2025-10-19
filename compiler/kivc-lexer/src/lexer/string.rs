//! String literal lexing with escape sequence support.

use super::cursor::LexerCursor;

/// Lexes a string literal, handling escape sequences
pub fn lex_string_literal(cursor: &mut LexerCursor) -> Result<String, String> {
    cursor.advance(); // consume opening "

    let mut string = String::new();

    loop {
        match cursor.current_char() {
            None => {
                return Err("unterminated string literal".to_string());
            }
            Some('"') => {
                cursor.advance(); // consume closing "
                return Ok(string);
            }
            Some('\\') => {
                cursor.advance();
                match cursor.current_char() {
                    Some('n') => {
                        string.push('\n');
                        cursor.advance();
                    }
                    Some('t') => {
                        string.push('\t');
                        cursor.advance();
                    }
                    Some('r') => {
                        string.push('\r');
                        cursor.advance();
                    }
                    Some('"') => {
                        string.push('"');
                        cursor.advance();
                    }
                    Some('\\') => {
                        string.push('\\');
                        cursor.advance();
                    }
                    Some(c) => {
                        return Err(format!("invalid escape sequence '\\{}'", c));
                    }
                    None => {
                        return Err("unterminated string literal".to_string());
                    }
                }
            }
            Some(c) => {
                string.push(c);
                cursor.advance();
            }
        }
    }
}
