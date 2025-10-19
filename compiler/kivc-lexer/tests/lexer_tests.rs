//! Integration tests for the lexer.

use kivc_lexer::{Lexer, TokenKind};
use kivc_span::SourceFile;

fn lex_source(source: &str) -> Vec<kivc_lexer::Token> {
    let file = SourceFile::new("test.kiv".to_string(), source.to_string());
    let mut lexer = Lexer::new(file);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        if token.kind == TokenKind::Eof {
            break;
        }
        tokens.push(token);
    }

    tokens
}

#[test]
fn test_keywords() {
    let tokens = lex_source("fun let mut const if else return");
    assert_eq!(tokens.len(), 7);
    assert!(matches!(tokens[0].kind, TokenKind::Fun));
    assert!(matches!(tokens[1].kind, TokenKind::Let));
    assert!(matches!(tokens[2].kind, TokenKind::Mut));
    assert!(matches!(tokens[3].kind, TokenKind::Const));
    assert!(matches!(tokens[4].kind, TokenKind::If));
    assert!(matches!(tokens[5].kind, TokenKind::Else));
    assert!(matches!(tokens[6].kind, TokenKind::Return));
}

#[test]
fn test_identifiers() {
    let tokens = lex_source("foo bar _baz x123");
    assert_eq!(tokens.len(), 4);
    assert!(matches!(tokens[0].kind, TokenKind::Ident(_)));
    assert!(matches!(tokens[1].kind, TokenKind::Ident(_)));
    assert!(matches!(tokens[2].kind, TokenKind::Ident(_)));
    assert!(matches!(tokens[3].kind, TokenKind::Ident(_)));
}

#[test]
fn test_integers() {
    let tokens = lex_source("0 42 1000");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].kind, TokenKind::IntLit(0));
    assert_eq!(tokens[1].kind, TokenKind::IntLit(42));
    assert_eq!(tokens[2].kind, TokenKind::IntLit(1000));
}

#[test]
fn test_floats() {
    let tokens = lex_source("2.71 0.5 123.456");
    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[0].kind, TokenKind::FloatLit(f) if (f - 2.71).abs() < 0.001));
    assert!(matches!(tokens[1].kind, TokenKind::FloatLit(f) if (f - 0.5).abs() < 0.001));
    assert!(matches!(tokens[2].kind, TokenKind::FloatLit(f) if (f - 123.456).abs() < 0.001));
}

#[test]
fn test_strings() {
    let tokens = lex_source(r#""hello" "world""#);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::TextLit("hello".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::TextLit("world".to_string()));
}

#[test]
fn test_string_escapes() {
    let tokens = lex_source(r#""hello\nworld" "tab\there""#);
    assert_eq!(tokens.len(), 2);
    assert_eq!(
        tokens[0].kind,
        TokenKind::TextLit("hello\nworld".to_string())
    );
    assert_eq!(tokens[1].kind, TokenKind::TextLit("tab\there".to_string()));
}

#[test]
fn test_operators() {
    let tokens = lex_source("= == != < <= > >= + - * /");
    assert_eq!(tokens.len(), 11);
    assert!(matches!(tokens[0].kind, TokenKind::Eq));
    assert!(matches!(tokens[1].kind, TokenKind::EqEq));
    assert!(matches!(tokens[2].kind, TokenKind::NotEq));
    assert!(matches!(tokens[3].kind, TokenKind::Lt));
    assert!(matches!(tokens[4].kind, TokenKind::Le));
    assert!(matches!(tokens[5].kind, TokenKind::Gt));
    assert!(matches!(tokens[6].kind, TokenKind::Ge));
    assert!(matches!(tokens[7].kind, TokenKind::Plus));
    assert!(matches!(tokens[8].kind, TokenKind::Minus));
    assert!(matches!(tokens[9].kind, TokenKind::Star));
    assert!(matches!(tokens[10].kind, TokenKind::Slash));
}

#[test]
fn test_delimiters() {
    let tokens = lex_source("( ) { } : , ;");
    assert_eq!(tokens.len(), 7);
    assert!(matches!(tokens[0].kind, TokenKind::LParen));
    assert!(matches!(tokens[1].kind, TokenKind::RParen));
    assert!(matches!(tokens[2].kind, TokenKind::LBrace));
    assert!(matches!(tokens[3].kind, TokenKind::RBrace));
    assert!(matches!(tokens[4].kind, TokenKind::Colon));
    assert!(matches!(tokens[5].kind, TokenKind::Comma));
    assert!(matches!(tokens[6].kind, TokenKind::Semicolon));
}

#[test]
fn test_comments() {
    let tokens = lex_source(
        r#"
        let x = 42; // this is a comment
        // another comment
        let y = 10;
    "#,
    );
    assert_eq!(tokens.len(), 10); // let x = 42 ; let y = 10 ;
    assert!(matches!(tokens[0].kind, TokenKind::Let));
    assert!(matches!(tokens[5].kind, TokenKind::Let));
}

#[test]
fn test_complete_expression() {
    let tokens = lex_source("let x: Int = 42 + 10;");
    assert_eq!(tokens.len(), 9);
    assert!(matches!(tokens[0].kind, TokenKind::Let));
    assert!(matches!(tokens[1].kind, TokenKind::Ident(_)));
    assert!(matches!(tokens[2].kind, TokenKind::Colon));
    assert!(matches!(tokens[3].kind, TokenKind::Ident(_))); // Int
    assert!(matches!(tokens[4].kind, TokenKind::Eq));
    assert_eq!(tokens[5].kind, TokenKind::IntLit(42));
    assert!(matches!(tokens[6].kind, TokenKind::Plus));
    assert_eq!(tokens[7].kind, TokenKind::IntLit(10));
    assert!(matches!(tokens[8].kind, TokenKind::Semicolon));
}
