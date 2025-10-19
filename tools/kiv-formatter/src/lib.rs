//! # Kiv Formatter Library
//!
//! This crate provides code formatting functionality for the Kiv programming language.

mod format;

pub use format::Formatter;

/// Formats Kiv source code
pub fn format_code(source: &str) -> Result<String, String> {
    use kivc_parser::Parser;
    use kivc_span::SourceFile;

    // Create source file
    let file = SourceFile::new("format".to_string(), source.to_string());

    // Parse
    let mut parser = Parser::new(file);
    let program = parser.parse_program();

    // Format
    let mut formatter = Formatter::new();
    Ok(formatter.format_program(&program))
}
