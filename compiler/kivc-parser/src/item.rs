//! Top-level item parsing (function definitions).

use crate::parser::Parser;
use crate::stmt::parse_block;
use crate::ty::parse_type;
use kivc_ast::{FunDef, Param};
use kivc_diagnostics::KivError;
use kivc_lexer::TokenKind;

/// Parses a function definition: `fun name(params) : RetType? { body }`
pub fn parse_function(parser: &mut Parser) -> Result<FunDef, KivError> {
    let start = parser.expect(TokenKind::Fun, "expected 'fun'")?;

    // Parse function name
    let name_token = parser.expect(
        TokenKind::Ident(String::new()),
        "expected function name after 'fun'",
    )?;
    let name = match &name_token.kind {
        TokenKind::Ident(s) => s.clone(),
        _ => unreachable!(),
    };

    // Parse parameter list
    parser.expect(TokenKind::LParen, "expected '(' after function name")?;

    let mut params = Vec::new();
    if !parser.check(&TokenKind::RParen) {
        loop {
            let param = parse_param(parser)?;
            params.push(param);

            if !parser.matches(&[TokenKind::Comma]) {
                break;
            }
        }
    }

    parser.expect(TokenKind::RParen, "expected ')' after parameters")?;

    // Optional return type
    let ret_ty = if parser.matches(&[TokenKind::Colon]) {
        Some(parse_type(parser)?)
    } else {
        None
    };

    // Parse body
    let body = parse_block(parser)?;

    let span = start.span.merge(&body.span).unwrap();

    Ok(FunDef {
        name,
        params,
        ret_ty,
        body,
        span,
    })
}

/// Parses a function parameter: `name : Type`
fn parse_param(parser: &mut Parser) -> Result<Param, KivError> {
    let name_token = parser.expect(TokenKind::Ident(String::new()), "expected parameter name")?;
    let name = match &name_token.kind {
        TokenKind::Ident(s) => s.clone(),
        _ => unreachable!(),
    };

    parser.expect(TokenKind::Colon, "expected ':' after parameter name")?;

    let ty = parse_type(parser)?;

    let span = name_token.span.merge(&ty.span).unwrap();

    Ok(Param { name, ty, span })
}
