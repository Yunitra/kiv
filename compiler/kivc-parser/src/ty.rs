//! Type annotation parsing.

use crate::parser::Parser;
use kivc_ast::Type;
use kivc_diagnostics::KivError;
use kivc_lexer::TokenKind;

/// Parses a type annotation
pub fn parse_type(parser: &mut Parser) -> Result<Type, KivError> {
    let token = parser.current().clone();

    match &token.kind {
        TokenKind::Ident(name) => {
            parser.advance();
            let kind = match name.as_str() {
                "Int" => kivc_ast::TypeKind::Int,
                "Float" => kivc_ast::TypeKind::Float,
                "Bool" => kivc_ast::TypeKind::Bool,
                "Text" => kivc_ast::TypeKind::Text,
                _ => {
                    return Err(parser.error(&token.span, &format!("unknown type '{}'", name)));
                }
            };
            Ok(Type::new(kind, token.span))
        }
        _ => Err(parser.error(&token.span, "expected type name")),
    }
}
