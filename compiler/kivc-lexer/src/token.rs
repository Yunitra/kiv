use kivc_span::Span;

/// A token represents a single lexical unit in the source code.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    /// Creates a new token
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// The kind/type of a token.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fun,
    Let,
    Mut,
    Const,
    If,
    Else,
    Return,
    True,
    False,

    // Literals
    IntLit(i64),
    FloatLit(f64),
    TextLit(String),

    // Operators
    Eq,    // =
    EqEq,  // ==
    NotEq, // !=
    Plus,  // +
    Minus, // -
    Star,  // *
    Slash, // /
    Lt,    // <
    Le,    // <=
    Gt,    // >
    Ge,    // >=

    // Delimiters
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    Colon,     // :
    Comma,     // ,
    Semicolon, // ;

    // Identifiers
    Ident(String),

    // Special
    Eof,
    Error,
}

impl TokenKind {
    /// Returns the string representation for display purposes
    pub fn as_str(&self) -> &str {
        match self {
            TokenKind::Fun => "fun",
            TokenKind::Let => "let",
            TokenKind::Mut => "mut",
            TokenKind::Const => "const",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::Return => "return",
            TokenKind::True => "true",
            TokenKind::False => "false",
            TokenKind::IntLit(_) => "integer literal",
            TokenKind::FloatLit(_) => "float literal",
            TokenKind::TextLit(_) => "text literal",
            TokenKind::Eq => "=",
            TokenKind::EqEq => "==",
            TokenKind::NotEq => "!=",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Lt => "<",
            TokenKind::Le => "<=",
            TokenKind::Gt => ">",
            TokenKind::Ge => ">=",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::Colon => ":",
            TokenKind::Comma => ",",
            TokenKind::Semicolon => ";",
            TokenKind::Ident(_) => "identifier",
            TokenKind::Eof => "end of file",
            TokenKind::Error => "error",
        }
    }

    /// Returns true if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Fun
                | TokenKind::Let
                | TokenKind::Mut
                | TokenKind::Const
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::Return
                | TokenKind::True
                | TokenKind::False
        )
    }
}

/// Converts a string to a keyword token kind, if it matches a keyword
pub fn keyword_or_ident(s: &str) -> TokenKind {
    match s {
        "fun" => TokenKind::Fun,
        "let" => TokenKind::Let,
        "mut" => TokenKind::Mut,
        "const" => TokenKind::Const,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "return" => TokenKind::Return,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        _ => TokenKind::Ident(s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivc_span::{SourceFile, Span};
    use std::sync::Arc;

    #[test]
    fn test_keyword_or_ident() {
        assert!(matches!(keyword_or_ident("fun"), TokenKind::Fun));
        assert!(matches!(keyword_or_ident("let"), TokenKind::Let));
        assert!(matches!(keyword_or_ident("foo"), TokenKind::Ident(_)));
    }

    #[test]
    fn test_is_keyword() {
        assert!(TokenKind::Fun.is_keyword());
        assert!(TokenKind::Let.is_keyword());
        assert!(!TokenKind::Ident("x".to_string()).is_keyword());
    }

    #[test]
    fn test_token_creation() {
        let file = SourceFile::new("test.kiv".to_string(), "let".to_string());
        let span = Span::new(Arc::clone(&file), 0.into(), 3.into());
        let token = Token::new(TokenKind::Let, span.clone());

        assert_eq!(token.kind, TokenKind::Let);
        assert_eq!(token.span.text(), "let");
    }
}
