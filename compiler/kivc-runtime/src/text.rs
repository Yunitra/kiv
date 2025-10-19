//! Copy-on-Write text type for Kiv.
//!
//! The `Text` type represents an immutable string that uses reference counting
//! for efficient memory management. Modifications create new instances (CoW).

use crate::rc::Rc;
use std::fmt;

/// An immutable, reference-counted string with Copy-on-Write semantics.
#[derive(Clone)]
pub struct Text {
    inner: Rc<TextInner>,
}

struct TextInner {
    data: std::vec::Vec<u8>,
}

impl Text {
    /// Creates a new `Text` from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            inner: Rc::new(TextInner {
                data: bytes.to_vec(),
            }),
        }
    }

    /// Creates a new `Text` from a string slice.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    /// Creates an empty `Text`.
    pub fn empty() -> Self {
        Self::from_bytes(&[])
    }

    /// Returns the length of the text in bytes.
    pub fn len(&self) -> usize {
        self.inner.data.len()
    }

    /// Returns `true` if the text is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.data.is_empty()
    }

    /// Returns the text as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner.data
    }

    /// Returns the text as a string slice, if it's valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.inner.data).ok()
    }

    /// Concatenates two `Text` values, creating a new instance.
    pub fn concat(&self, other: &Text) -> Self {
        let mut new_data = self.inner.data.clone();
        new_data.extend_from_slice(&other.inner.data);
        Self {
            inner: Rc::new(TextInner { data: new_data }),
        }
    }

    /// Compares two `Text` values for equality.
    pub fn equals(&self, other: &Text) -> bool {
        self.inner.data == other.inner.data
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Some(s) => write!(f, "{}", s),
            None => write!(f, "<invalid UTF-8>"),
        }
    }
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Some(s) => write!(f, "Text({:?})", s),
            None => write!(f, "Text(<invalid UTF-8>)"),
        }
    }
}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl Eq for Text {}

// C ABI exports for runtime

/// Creates a new Text from a C string (null-terminated).
///
/// # Safety
/// The pointer must be valid and point to a null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kiv_text_from_cstr(ptr: *const u8) -> *mut Text {
    unsafe {
        if ptr.is_null() {
            return std::ptr::null_mut();
        }

        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }

        let slice = std::slice::from_raw_parts(ptr, len);
        let text = Text::from_bytes(slice);
        Box::into_raw(Box::new(text))
    }
}

/// Creates a new Text from bytes with a known length.
///
/// # Safety
/// The pointer must be valid and point to at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kiv_text_from_bytes(ptr: *const u8, len: usize) -> *mut Text {
    unsafe {
        if ptr.is_null() {
            return std::ptr::null_mut();
        }

        let slice = std::slice::from_raw_parts(ptr, len);
        let text = Text::from_bytes(slice);
        Box::into_raw(Box::new(text))
    }
}

/// Clones a Text, incrementing its reference count.
///
/// # Safety
/// The pointer must be a valid Text pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kiv_text_clone(text: *const Text) -> *mut Text {
    unsafe {
        if text.is_null() {
            return std::ptr::null_mut();
        }

        let text_ref = &*text;
        Box::into_raw(Box::new(text_ref.clone()))
    }
}

/// Drops a Text, decrementing its reference count.
///
/// # Safety
/// The pointer must be a valid Text pointer obtained from a runtime function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kiv_text_drop(text: *mut Text) {
    unsafe {
        if !text.is_null() {
            let _ = Box::from_raw(text);
        }
    }
}

/// Returns the length of a Text in bytes.
///
/// # Safety
/// The pointer must be a valid Text pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kiv_text_len(text: *const Text) -> usize {
    unsafe {
        if text.is_null() {
            return 0;
        }
        (*text).len()
    }
}

/// Concatenates two Text values.
///
/// # Safety
/// Both pointers must be valid Text pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kiv_text_concat(left: *const Text, right: *const Text) -> *mut Text {
    unsafe {
        if left.is_null() || right.is_null() {
            return std::ptr::null_mut();
        }

        let result = (*left).concat(&*right);
        Box::into_raw(Box::new(result))
    }
}

/// Compares two Text values for equality.
///
/// # Safety
/// Both pointers must be valid Text pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kiv_text_equals(left: *const Text, right: *const Text) -> bool {
    unsafe {
        if left.is_null() || right.is_null() {
            return false;
        }
        (*left).equals(&*right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_basic() {
        let text = Text::from_str("Hello, Kiv!");
        assert_eq!(text.len(), 11);
        assert_eq!(text.as_str(), Some("Hello, Kiv!"));
    }

    #[test]
    fn test_text_empty() {
        let text = Text::empty();
        assert!(text.is_empty());
        assert_eq!(text.len(), 0);
    }

    #[test]
    fn test_text_clone() {
        let text1 = Text::from_str("test");
        let text2 = text1.clone();
        assert_eq!(text1.as_str(), text2.as_str());
    }

    #[test]
    fn test_text_concat() {
        let text1 = Text::from_str("Hello");
        let text2 = Text::from_str(", World!");
        let result = text1.concat(&text2);
        assert_eq!(result.as_str(), Some("Hello, World!"));
    }

    #[test]
    fn test_text_equals() {
        let text1 = Text::from_str("same");
        let text2 = Text::from_str("same");
        let text3 = Text::from_str("different");

        assert!(text1.equals(&text2));
        assert!(!text1.equals(&text3));
    }
}
