//! Small String Optimization (SSO) for the Text type.
//!
//! For simplicity and following YAGNI, this implementation supports
//! only inline storage for strings up to 23 bytes.

use std::fmt;

const INLINE_CAP: usize = 23;

/// A text type with inline storage for strings up to 23 bytes
#[repr(C)]
pub struct Text {
    bytes: [u8; INLINE_CAP],
    len: u8,
}

impl Text {
    pub fn new(s: &str) -> Self {
        let len = s.len();
        assert!(len <= INLINE_CAP, "String too long: {} > {}", len, INLINE_CAP);
        
        let mut bytes = [0u8; INLINE_CAP];
        bytes[..len].copy_from_slice(s.as_bytes());
        
        Self { bytes, len: len as u8 }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            let slice = &self.bytes[..self.len as usize];
            std::str::from_utf8_unchecked(slice)
        }
    }

    pub fn concat(&self, other: &Self) -> Self {
        let mut s = String::with_capacity(self.len() + other.len());
        s.push_str(self.as_str());
        s.push_str(other.as_str());
        Self::new(&s)
    }

    pub fn push(&mut self, ch: char) {
        let mut s = self.as_str().to_string();
        s.push(ch);
        *self = Self::new(&s);
    }
}

impl Clone for Text {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes,
            len: self.len,
        }
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Text({:?})", self.as_str())
    }
}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Text {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sso_inline() {
        let text = Text::new("hello");
        assert_eq!(text.as_str(), "hello");
        assert_eq!(text.len(), 5);
    }

    #[test]
    fn test_sso_clone() {
        let text1 = Text::new("hello");
        let text2 = text1.clone();
        assert_eq!(text1.as_str(), text2.as_str());
    }

    #[test]
    fn test_sso_concat() {
        let text1 = Text::new("hello");
        let text2 = Text::new(" world");
        let result = text1.concat(&text2);
        assert_eq!(result.as_str(), "hello world");
    }

    #[test]
    fn test_sso_push() {
        let mut text = Text::new("hello");
        text.push('!');
        assert_eq!(text.as_str(), "hello!");
    }

    #[test]
    fn test_sso_empty() {
        let text = Text::new("");
        assert!(text.is_empty());
    }

    #[test]
    fn test_sso_boundary() {
        let text23 = Text::new("12345678901234567890123");
        assert_eq!(text23.len(), 23);
    }

    #[test]
    #[should_panic(expected = "String too long")]
    fn test_sso_too_long() {
        Text::new("123456789012345678901234"); // 24 bytes
    }
}
