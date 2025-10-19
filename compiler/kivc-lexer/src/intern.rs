//! String interning for identifiers and string literals.
//!
//! This module provides a string interning system to reduce memory usage
//! and improve performance by storing only one copy of each unique string.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// An interned string - a cheap-to-clone handle to a unique string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InternedString(Arc<str>);

impl InternedString {
    /// Returns the string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the length
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&str> for InternedString {
    fn from(s: &str) -> Self {
        Interner::global().intern(s)
    }
}

impl From<String> for InternedString {
    fn from(s: String) -> Self {
        Interner::global().intern(&s)
    }
}

impl std::fmt::Display for InternedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for InternedString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// String interner that stores unique strings
pub struct Interner {
    map: Mutex<HashMap<Arc<str>, ()>>,
}

impl Interner {
    /// Creates a new interner
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the global interner instance
    pub fn global() -> &'static Self {
        static INTERNER: OnceLock<Interner> = OnceLock::new();
        INTERNER.get_or_init(Interner::new)
    }

    /// Interns a string, returning a cheap-to-clone handle
    pub fn intern(&self, s: &str) -> InternedString {
        let mut map = self.map.lock().unwrap();

        // Try to find existing string
        if let Some((key, _)) = map.get_key_value(s as &str) {
            return InternedString(Arc::clone(key));
        }

        // Create new Arc<str> and insert
        let arc: Arc<str> = s.into();
        map.insert(Arc::clone(&arc), ());
        InternedString(arc)
    }

    /// Returns the number of interned strings
    pub fn len(&self) -> usize {
        self.map.lock().unwrap().len()
    }

    /// Returns true if no strings are interned
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_same_string() {
        let interner = Interner::new();
        let s1 = interner.intern("hello");
        let s2 = interner.intern("hello");

        // Same underlying Arc
        assert_eq!(s1, s2);
        assert_eq!(Arc::as_ptr(&s1.0), Arc::as_ptr(&s2.0));
    }

    #[test]
    fn test_intern_different_strings() {
        let interner = Interner::new();
        let s1 = interner.intern("hello");
        let s2 = interner.intern("world");

        assert_ne!(s1, s2);
        assert_ne!(Arc::as_ptr(&s1.0), Arc::as_ptr(&s2.0));
    }

    #[test]
    fn test_interned_string_display() {
        let s = InternedString::from("test");
        assert_eq!(format!("{}", s), "test");
    }

    #[test]
    fn test_interned_string_as_str() {
        let s = InternedString::from("hello");
        assert_eq!(s.as_str(), "hello");
    }
}
