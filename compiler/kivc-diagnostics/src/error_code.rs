use std::fmt;

/// An error code in the format `kiv::category::E001`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErrorCode {
    category: &'static str,
    number: u16,
}

impl ErrorCode {
    /// Creates a new error code
    pub const fn new(category: &'static str, number: u16) -> Self {
        Self { category, number }
    }

    /// Returns the error category
    pub fn category(&self) -> &'static str {
        self.category
    }

    /// Returns the error number
    pub fn number(&self) -> u16 {
        self.number
    }

    /// Formats the error code as a string
    pub fn as_str(&self) -> String {
        format!("kiv::{}::E{:03}", self.category, self.number)
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "kiv::{}::E{:03}", self.category, self.number)
    }
}

// Predefined error codes by category

// Syntax errors (E001-E099)
pub const E001_UNEXPECTED_TOKEN: ErrorCode = ErrorCode::new("syntax", 1);
pub const E002_EXPECTED_TOKEN: ErrorCode = ErrorCode::new("syntax", 2);
pub const E003_INVALID_LITERAL: ErrorCode = ErrorCode::new("syntax", 3);
pub const E004_UNCLOSED_DELIMITER: ErrorCode = ErrorCode::new("syntax", 4);
pub const E005_UNEXPECTED_EOF: ErrorCode = ErrorCode::new("syntax", 5);

// Type errors (E100-E199)
pub const E100_TYPE_MISMATCH: ErrorCode = ErrorCode::new("type", 100);
pub const E101_UNDEFINED_VARIABLE: ErrorCode = ErrorCode::new("type", 101);
pub const E102_DUPLICATE_DEFINITION: ErrorCode = ErrorCode::new("type", 102);
pub const E103_IMMUTABLE_ASSIGN: ErrorCode = ErrorCode::new("type", 103);
pub const E104_INVALID_OPERATION: ErrorCode = ErrorCode::new("type", 104);
pub const E105_UNDEFINED_FUNCTION: ErrorCode = ErrorCode::new("type", 105);
pub const E106_WRONG_ARG_COUNT: ErrorCode = ErrorCode::new("type", 106);
pub const E107_INVALID_RETURN: ErrorCode = ErrorCode::new("type", 107);

// IO errors (E200-E299)
pub const E200_FILE_NOT_FOUND: ErrorCode = ErrorCode::new("io", 200);
pub const E201_READ_ERROR: ErrorCode = ErrorCode::new("io", 201);
pub const E202_WRITE_ERROR: ErrorCode = ErrorCode::new("io", 202);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_format() {
        assert_eq!(E001_UNEXPECTED_TOKEN.as_str(), "kiv::syntax::E001");
        assert_eq!(E100_TYPE_MISMATCH.as_str(), "kiv::type::E100");
        assert_eq!(E200_FILE_NOT_FOUND.as_str(), "kiv::io::E200");
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(format!("{}", E001_UNEXPECTED_TOKEN), "kiv::syntax::E001");
    }

    #[test]
    fn test_error_code_accessors() {
        assert_eq!(E001_UNEXPECTED_TOKEN.category(), "syntax");
        assert_eq!(E001_UNEXPECTED_TOKEN.number(), 1);
    }
}
