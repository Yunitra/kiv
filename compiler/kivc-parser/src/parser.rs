//! Main parser structure and token stream management.

use crate::item::parse_function;
use crate::recovery::synchronize;
use kivc_ast::Program;
use kivc_diagnostics::{DiagnosticsCollector, KivError};
use kivc_lexer::{Lexer, Token, TokenKind};
use kivc_span::{SourceFile, Span};
use std::sync::Arc;

/// A recursive descent parser that produces an AST from tokens.
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    diagnostics: DiagnosticsCollector,
}

impl Parser {
    /// Creates a new parser from a source file
    pub fn new(file: Arc<SourceFile>) -> Self {
        let mut lexer = Lexer::new(file);
        let mut tokens = Vec::new();

        loop {
            let token = lexer.next_token();
            if token.kind == TokenKind::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }

        let diagnostics = lexer.into_diagnostics();

        // If there are lexical errors, add them to diagnostics
        Self {
            tokens,
            current: 0,
            diagnostics,
        }
    }

    /// Parses the entire program (list of function definitions)
    pub fn parse_program(&mut self) -> Program {
        let mut functions = Vec::new();

        while !self.is_at_end() {
            match parse_function(self) {
                Ok(func) => functions.push(func),
                Err(e) => {
                    self.diagnostics.add(e);
                    synchronize(self);
                }
            }
        }

        Program { functions }
    }

    /// Returns the collected diagnostics
    pub fn diagnostics(&self) -> &DiagnosticsCollector {
        &self.diagnostics
    }

    /// Consumes the parser and returns the diagnostics
    pub fn into_diagnostics(self) -> DiagnosticsCollector {
        self.diagnostics
    }

    /// Returns the current token without consuming it
    pub(crate) fn current(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Peeks ahead by `n` tokens
    #[allow(dead_code)]
    pub(crate) fn peek(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.current + n)
    }

    /// Checks if we're at the end of input
    pub(crate) fn is_at_end(&self) -> bool {
        self.current().kind == TokenKind::Eof
    }

    /// Advances to the next token and returns the previous one
    pub(crate) fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens[self.current - 1].clone()
    }

    /// Checks if the current token matches the given kind
    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    /// Consumes the current token if it matches the given kind
    pub(crate) fn matches(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Expects the current token to be of the given kind, advances if so
    pub(crate) fn expect(&mut self, kind: TokenKind, message: &str) -> Result<Token, KivError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            let span = self.current().span.clone();
            Err(KivError::syntax(
                &span,
                message,
                "unexpected token",
                None,
                kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
            ))
        }
    }

    /// Reports a parser error
    pub(crate) fn error(&mut self, span: &Span, message: &str) -> KivError {
        KivError::syntax(
            span,
            message,
            "here",
            None,
            kivc_diagnostics::error_code::E001_UNEXPECTED_TOKEN,
        )
    }
}
