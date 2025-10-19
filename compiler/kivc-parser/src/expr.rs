//! Expression parsing using Pratt parsing (operator precedence).

use crate::parser::Parser;
use crate::stmt::parse_block;
use kivc_ast::{BinOp, Expr, ExprKind, Literal};
use kivc_diagnostics::KivError;
use kivc_lexer::TokenKind;

/// Operator precedence levels (lower number = lower precedence)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None = 0,
    Assignment = 1, // =
    Comparison = 2, // ==, !=, <, <=, >, >=
    AddSub = 3,     // +, -
    MulDiv = 4,     // *, /
    Call = 5,       // (), primary
}

/// Parses an expression with the given precedence
pub fn parse_expr(parser: &mut Parser) -> Result<Expr, KivError> {
    parse_expr_with_precedence(parser, Precedence::Assignment)
}

/// Pratt parsing: parses expression with minimum precedence
fn parse_expr_with_precedence(parser: &mut Parser, min_prec: Precedence) -> Result<Expr, KivError> {
    let mut left = parse_primary(parser)?;

    while !parser.is_at_end() {
        let prec = get_infix_precedence(&parser.current().kind);
        if prec < min_prec {
            break;
        }

        left = parse_infix(parser, left, prec)?;
    }

    Ok(left)
}

/// Parses a primary expression (literals, identifiers, calls, if, parenthesized)
fn parse_primary(parser: &mut Parser) -> Result<Expr, KivError> {
    let token = parser.current().clone();

    match &token.kind {
        // Integer literal
        TokenKind::IntLit(val) => {
            parser.advance();
            Ok(Expr::literal(Literal::Int(*val), token.span))
        }

        // Float literal
        TokenKind::FloatLit(val) => {
            parser.advance();
            Ok(Expr::literal(Literal::Float(*val), token.span))
        }

        // Text literal
        TokenKind::TextLit(s) => {
            parser.advance();
            Ok(Expr::literal(Literal::Text(s.clone()), token.span))
        }

        // Boolean literals (true/false are keywords)
        TokenKind::True => {
            parser.advance();
            Ok(Expr::literal(Literal::Bool(true), token.span))
        }
        TokenKind::False => {
            parser.advance();
            Ok(Expr::literal(Literal::Bool(false), token.span))
        }

        // Identifier (variable or function call)
        TokenKind::Ident(name) => {
            parser.advance();

            // Check if it's a function call
            if parser.check(&TokenKind::LParen) {
                parse_call(parser, name.clone(), token.span)
            } else {
                Ok(Expr::var(name.clone(), token.span))
            }
        }

        // If expression
        TokenKind::If => parse_if_expr(parser),

        // Parenthesized expression
        TokenKind::LParen => {
            parser.advance(); // consume '('
            let expr = parse_expr(parser)?;
            parser.expect(TokenKind::RParen, "expected ')' after expression")?;
            Ok(expr)
        }

        _ => Err(parser.error(&token.span, "expected expression")),
    }
}

/// Parses an infix binary operator
fn parse_infix(parser: &mut Parser, left: Expr, prec: Precedence) -> Result<Expr, KivError> {
    let op_token = parser.current().clone();

    // Handle assignment separately
    if op_token.kind == TokenKind::Eq && prec == Precedence::Assignment {
        parser.advance();
        let value = parse_expr_with_precedence(parser, Precedence::Assignment)?;

        // Extract variable name from left side
        let target = match &left.kind {
            ExprKind::Var { name } => name.clone(),
            _ => {
                return Err(parser.error(&left.span, "invalid assignment target"));
            }
        };

        let span = left.span.merge(&value.span).unwrap();

        return Ok(Expr::assign(target, value, span));
    }

    // Parse binary operators
    let op = match &op_token.kind {
        TokenKind::Plus => BinOp::Add,
        TokenKind::Minus => BinOp::Sub,
        TokenKind::Star => BinOp::Mul,
        TokenKind::Slash => BinOp::Div,
        TokenKind::EqEq => BinOp::Eq,
        TokenKind::NotEq => BinOp::NotEq,
        TokenKind::Lt => BinOp::Lt,
        TokenKind::Le => BinOp::Le,
        TokenKind::Gt => BinOp::Gt,
        TokenKind::Ge => BinOp::Ge,
        _ => return Err(parser.error(&op_token.span, "unexpected operator")),
    };

    parser.advance();

    // Right-associative: use same precedence
    // Left-associative: use next higher precedence
    let next_prec = Precedence::from(prec as u8 + 1);
    let right = parse_expr_with_precedence(parser, next_prec)?;

    let span = left.span.merge(&right.span).unwrap();

    Ok(Expr::binary(op, left, right, span))
}

/// Returns the precedence of an infix operator
fn get_infix_precedence(kind: &TokenKind) -> Precedence {
    match kind {
        TokenKind::Eq => Precedence::Assignment,
        TokenKind::EqEq
        | TokenKind::NotEq
        | TokenKind::Lt
        | TokenKind::Le
        | TokenKind::Gt
        | TokenKind::Ge => Precedence::Comparison,
        TokenKind::Plus | TokenKind::Minus => Precedence::AddSub,
        TokenKind::Star | TokenKind::Slash => Precedence::MulDiv,
        _ => Precedence::None,
    }
}

/// Parses a function call: `name(args)`
fn parse_call(
    parser: &mut Parser,
    func: String,
    start_span: kivc_span::Span,
) -> Result<Expr, KivError> {
    parser.expect(TokenKind::LParen, "expected '('")?;

    let mut args = Vec::new();
    if !parser.check(&TokenKind::RParen) {
        loop {
            args.push(parse_expr(parser)?);
            if !parser.matches(&[TokenKind::Comma]) {
                break;
            }
        }
    }

    let end = parser.expect(TokenKind::RParen, "expected ')' after arguments")?;

    let span = start_span.merge(&end.span).unwrap();

    Ok(Expr::call(func, args, span))
}

/// Parses an if expression: `if cond { then } else { else }`
fn parse_if_expr(parser: &mut Parser) -> Result<Expr, KivError> {
    let start = parser.advance().span; // consume 'if'

    let cond = parse_expr(parser)?;
    let then_branch = parse_block(parser)?;

    let else_branch = if parser.matches(&[TokenKind::Else]) {
        Some(parse_block(parser)?)
    } else {
        None
    };

    let span = if let Some(ref eb) = else_branch {
        start.merge(&eb.span).unwrap()
    } else {
        start.merge(&then_branch.span).unwrap()
    };

    Ok(Expr::if_expr(cond, then_branch, else_branch, span))
}

impl From<u8> for Precedence {
    fn from(val: u8) -> Self {
        match val {
            0 => Precedence::None,
            1 => Precedence::Assignment,
            2 => Precedence::Comparison,
            3 => Precedence::AddSub,
            4 => Precedence::MulDiv,
            5 => Precedence::Call,
            _ => Precedence::Call,
        }
    }
}
