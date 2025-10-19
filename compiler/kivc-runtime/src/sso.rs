//! Small String Optimization (SSO) for the Text type.
//!
//! Strings ≤ 23 bytes stored inline, larger strings use Arc for safety.

use std::fmt;
use std::sync::Arc;

const INLINE_CAP: usize = 23;

/// A Copy-on-Write text type with SSO
pub struct Text {
    repr: TextRepr,
}

enum TextRepr {
    Inline { bytes: [u8; INLINE_CAP], len: u8 },
    Heap(Arc<String>),
}

impl Text {
    pub fn new(s: &str) -> Self {
        let len = s.len();
        
        if len <= INLINE_CAP {
            let mut bytes = [0u8; INLINE_CAP];
            bytes[..len].copy_from_slice(s.as_bytes());
            Self {
                repr: TextRepr::Inline {
                    bytes,
                    len: len as u8,
                },
            }
        } else {
            Self {
                repr: TextRepr::Heap(Arc::new(s.to_string())),
            }
        }
    }
    
    pub fn is_inline(&self) -> bool {
        matches!(self.repr, TextRepr::Inline { .. })
    }
    
    pub fn len(&self) -> usize {
        match &self.repr {
            TextRepr::Inline { len, .. } => *len as usize,
            TextRepr::Heap(s) => s.len(),
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    pub fn as_str(&self) -> &str {
        match &self.repr {
            TextRepr::Inline { bytes, len } => {
                unsafe { std::str::from_utf8_unchecked(&bytes[..*len as usize]) }
            }
            TextRepr::Heap(s) => s.as_str(),
        }
    }
    
    pub fn ref_count(&self) -> usize {
        match &self.repr {
            TextRepr::Inline { .. } => 1,
            TextRepr::Heap(arc) => Arc::strong_count(arc),
        }
    }
    
    fn make_unique(&mut self) {
        if let TextRepr::Heap(arc) = &self.repr {
            if Arc::strong_count(arc) > 1 {
                let s = arc.as_str().to_string();
                *self = Self::new(&s);
            }
        }
    }
    
    pub fn concat(&self, other: &Self) -> Self {
        let mut s = String::with_capacity(self.len() + other.len());
        s.push_str(self.as_str());
        s.push_str(other.as_str());
        Self::new(&s)
    }
    
    pub fn push(&mut self, ch: char) {
        self.make_unique();
        let mut s = self.as_str().to_string();
        s.push(ch);
        *self = Self::new(&s);
    }
}

impl Clone for Text {
    fn clone(&self) -> Self {
        match &self.repr {
            TextRepr::Inline { bytes, len } => Self {
                repr: TextRepr::Inline {
                    bytes: *bytes,
                    len: *len,
                },
            },
            TextRepr::Heap(arc) => Self {
                repr: TextRepr::Heap(Arc::clone(arc)),
            },
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
        write!(f, "Text({:?}, inline={})", self.as_str(), self.is_inline())
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
        assert!(text.is_inline());
        assert_eq!(text.as_str(), "hello");
        assert_eq!(text.len(), 5);
    }
    
    #[test]
    fn test_sso_heap() {
        let long_str = "this is a very long string that exceeds inline capacity";
        let text = Text::new(long_str);
        assert!(!text.is_inline());
        assert_eq!(text.as_str(), long_str);
    }
    
    #[test]
    fn test_sso_clone_inline() {
        let text1 = Text::new("hello");
        let text2 = text1.clone();
        assert_eq!(text1.as_str(), text2.as_str());
        assert!(text1.is_inline());
        assert!(text2.is_inline());
    }
    
    #[test]
    fn test_sso_clone_heap() {
        let long_str = "this is a very long string that exceeds inline capacity";
        let text1 = Text::new(long_str);
        let text2 = text1.clone();
        assert_eq!(text1.as_str(), text2.as_str());
        assert_eq!(text1.ref_count(), 2);
        assert_eq!(text2.ref_count(), 2);
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
    fn test_sso_cow_on_write() {
        let long_str = "this is a very long string that exceeds inline capacity";
        let text1 = Text::new(long_str);
        let mut text2 = text1.clone();
        
        assert_eq!(text1.ref_count(), 2);
        assert_eq!(text2.ref_count(), 2);
        
        text2.push('!');
        
        assert_eq!(text1.ref_count(), 1);
        assert_eq!(text2.ref_count(), 1);
        assert_eq!(text1.as_str(), long_str);
        assert_eq!(text2.as_str(), &format!("{}!", long_str));
    }
    
    #[test]
    fn test_sso_empty() {
        let text = Text::new("");
        assert!(text.is_empty());
        assert!(text.is_inline());
    }
    
    #[test]
    fn test_sso_boundary() {
        let text23 = Text::new("12345678901234567890123");
        assert!(text23.is_inline());
        assert_eq!(text23.len(), 23);
        
        let text24 = Text::new("123456789012345678901234");
        assert!(!text24.is_inline());
        assert_eq!(text24.len(), 24);
    }
}
