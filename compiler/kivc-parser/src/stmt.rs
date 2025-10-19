//! Statement parsing (let, const, return, expr).

use crate::expr::parse_expr;
use crate::parser::Parser;
use crate::ty::parse_type;
use kivc_ast::{Block, Stmt, StmtKind};
use kivc_diagnostics::KivError;
use kivc_lexer::TokenKind;
use kivc_span::Span;

/// Helper to consume optional semicolon and return appropriate span
fn consume_optional_semicolon(parser: &mut Parser, base_span: Span) -> Span {
    if parser.check(&TokenKind::Semicolon) {
        let semi = parser.advance().span;
        base_span.merge(&semi).unwrap_or(base_span)
    } else {
        base_span
    }
}

/// Parses a statement
pub fn parse_stmt(parser: &mut Parser) -> Result<Stmt, KivError> {
    match parser.current().kind {
        TokenKind::Let => parse_let_stmt(parser),
        TokenKind::Const => parse_const_stmt(parser),
        TokenKind::Return => parse_return_stmt(parser),
        _ => parse_expr_stmt(parser),
    }
}

/// Parses a let statement: `let mut? name : Type? = expr;`
fn parse_let_stmt(parser: &mut Parser) -> Result<Stmt, KivError> {
    let start = parser.advance().span; // consume 'let'

    // Check for 'mut'
    let mutable = parser.matches(&[TokenKind::Mut]);

    // Parse identifier
    let name_token = parser.expect(
        TokenKind::Ident(String::new()),
        "expected variable name after 'let'",
    )?;
    let name = match &name_token.kind {
        TokenKind::Ident(s) => s.clone(),
        _ => unreachable!(),
    };

    // Optional type annotation
    let ty = if parser.matches(&[TokenKind::Colon]) {
        Some(parse_type(parser)?)
    } else {
        None
    };

    // Expect '='
    parser.expect(TokenKind::Eq, "expected '=' in let statement")?;

    // Parse initializer expression
    let init = parse_expr(parser)?;

    // Optional semicolon
    let span = consume_optional_semicolon(parser, start.merge(&init.span).unwrap());

    Ok(Stmt {
        kind: StmtKind::Let {
            mutable,
            name,
            ty,
            init,
        },
        span,
    })
}

/// Parses a const statement: `const name = expr;`
fn parse_const_stmt(parser: &mut Parser) -> Result<Stmt, KivError> {
    let start = parser.advance().span; // consume 'const'

    // Parse identifier
    let name_token = parser.expect(
        TokenKind::Ident(String::new()),
        "expected constant name after 'const'",
    )?;
    let name = match &name_token.kind {
        TokenKind::Ident(s) => s.clone(),
        _ => unreachable!(),
    };

    // Expect '='
    parser.expect(TokenKind::Eq, "expected '=' in const statement")?;

    // Parse value expression
    let value = parse_expr(parser)?;

    // Optional semicolon
    let span = consume_optional_semicolon(parser, start.merge(&value.span).unwrap());

    Ok(Stmt {
        kind: StmtKind::Const { name, value },
        span,
    })
}

/// Parses a return statement: `return expr?;`
fn parse_return_stmt(parser: &mut Parser) -> Result<Stmt, KivError> {
    let start = parser.advance().span; // consume 'return'

    // Optional return value
    let (value, value_span) = if parser.check(&TokenKind::Semicolon)
        || parser.check(&TokenKind::RBrace)
        || parser.is_at_end()
    {
        (None, start)
    } else {
        let expr = parse_expr(parser)?;
        let span = start.merge(&expr.span).unwrap();
        (Some(expr), span)
    };

    // Optional semicolon
    let span = consume_optional_semicolon(parser, value_span);

    Ok(Stmt {
        kind: StmtKind::Return { value },
        span,
    })
}

/// Parses an expression statement: `expr;` (semicolon optional)
fn parse_expr_stmt(parser: &mut Parser) -> Result<Stmt, KivError> {
    let expr = parse_expr(parser)?;

    // Optional semicolon (clone span before moving expr)
    let expr_span = expr.span.clone();
    let span = consume_optional_semicolon(parser, expr_span);

    Ok(Stmt {
        kind: StmtKind::Expr { expr },
        span,
    })
}

/// Parses a block: `{ stmt* }`
pub fn parse_block(parser: &mut Parser) -> Result<Block, KivError> {
    let start = parser.expect(TokenKind::LBrace, "expected '{'")?;

    let mut stmts = Vec::new();

    while !parser.check(&TokenKind::RBrace) && !parser.is_at_end() {
        match parse_stmt(parser) {
            Ok(stmt) => stmts.push(stmt),
            Err(_e) => {
                // Try to recover from parse error
                while !parser.check(&TokenKind::Semicolon)
                    && !parser.check(&TokenKind::RBrace)
                    && !parser.is_at_end()
                {
                    parser.advance();
                }
                if parser.check(&TokenKind::Semicolon) {
                    parser.advance();
                }
            }
        }
    }

    let end = parser.expect(TokenKind::RBrace, "expected '}' to close block")?;

    let span = start.span.merge(&end.span).unwrap();

    Ok(Block { stmts, span })
}
