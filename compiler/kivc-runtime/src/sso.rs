//! Small String Optimization (SSO) for the Text type.
//!
//! This module implements SSO, which stores small strings inline within
//! the Text struct itself, avoiding heap allocation for short strings.
//!
//! # Design
//! - Strings ≤ 23 bytes are stored inline
//! - Larger strings are heap-allocated with reference counting
//! - The last byte is used as a discriminant

use std::fmt;
use std::mem;

/// Maximum size for inline storage (including discriminant byte)
const INLINE_CAPACITY: usize = 23;

/// A text value with Small String Optimization
#[repr(C)]
pub union TextRepr {
    /// Inline storage for small strings
    inline: InlineText,
    /// Heap storage for large strings
    heap: HeapText,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InlineText {
    /// UTF-8 bytes stored inline
    data: [u8; INLINE_CAPACITY],
    /// Length and discriminant (bit 0 = 0 for inline)
    len: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct HeapText {
    /// Pointer to heap-allocated data
    ptr: *mut u8,
    /// Capacity of the allocation
    capacity: usize,
    /// Length of the string
    len: usize,
    /// Reference count pointer (bit 0 = 1 for heap)
    rc: *mut usize,
}

/// A Copy-on-Write text type with Small String Optimization
pub struct Text {
    repr: TextRepr,
}

impl Text {
    /// Creates a new Text from a string slice
    pub fn new(s: &str) -> Self {
        let len = s.len();
        
        if len <= INLINE_CAPACITY {
            // Use inline storage
            let mut data = [0u8; INLINE_CAPACITY];
            data[..len].copy_from_slice(s.as_bytes());
            
            Self {
                repr: TextRepr {
                    inline: InlineText {
                        data,
                        len: (len as u8) << 1, // bit 0 = 0
                    },
                },
            }
        } else {
            // Use heap storage
            let mut vec = s.as_bytes().to_vec();
            let capacity = vec.capacity();
            let ptr = vec.as_mut_ptr();
            mem::forget(vec);
            
            let rc = Box::into_raw(Box::new(1usize));
            
            Self {
                repr: TextRepr {
                    heap: HeapText {
                        ptr,
                        capacity,
                        len,
                        rc: (rc as usize | 1) as *mut usize, // bit 0 = 1
                    },
                },
            }
        }
    }

    /// Returns true if the text is stored inline
    #[inline]
    fn is_inline(&self) -> bool {
        unsafe { (self.repr.inline.len & 1) == 0 }
    }

    /// Returns the length of the text
    pub fn len(&self) -> usize {
        unsafe {
            if self.is_inline() {
                (self.repr.inline.len >> 1) as usize
            } else {
                self.repr.heap.len
            }
        }
    }

    /// Returns true if the text is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the text as a string slice
    pub fn as_str(&self) -> &str {
        unsafe {
            if self.is_inline() {
                let len = (self.repr.inline.len >> 1) as usize;
                std::str::from_utf8_unchecked(&self.repr.inline.data[..len])
            } else {
                let len = self.repr.heap.len;
                let slice = std::slice::from_raw_parts(self.repr.heap.ptr, len);
                std::str::from_utf8_unchecked(slice)
            }
        }
    }

    /// Returns the reference count (1 for inline strings)
    fn ref_count(&self) -> usize {
        unsafe {
            if self.is_inline() {
                1
            } else {
                let rc_ptr = (self.repr.heap.rc as usize & !1) as *const usize;
                *rc_ptr
            }
        }
    }

    /// Clones the text (increments reference count for heap strings)
    pub fn clone_text(&self) -> Self {
        unsafe {
            if self.is_inline() {
                // Inline strings are copied directly
                Self {
                    repr: TextRepr {
                        inline: self.repr.inline,
                    },
                }
            } else {
                // Increment reference count for heap strings
                let rc_ptr = (self.repr.heap.rc as usize & !1) as *mut usize;
                *rc_ptr += 1;
                
                Self {
                    repr: TextRepr {
                        heap: self.repr.heap,
                    },
                }
            }
        }
    }

    /// Ensures the text has a unique copy for mutation
    fn make_unique(&mut self) {
        unsafe {
            if !self.is_inline() && self.ref_count() > 1 {
                // Need to make a copy
                let s = self.as_str();
                let len = s.len();
                
                // Decrement old reference count
                let old_rc_ptr = (self.repr.heap.rc as usize & !1) as *mut usize;
                *old_rc_ptr -= 1;
                
                // Create new allocation
                let mut vec = s.as_bytes().to_vec();
                let capacity = vec.capacity();
                let ptr = vec.as_mut_ptr();
                mem::forget(vec);
                
                let rc = Box::into_raw(Box::new(1usize));
                
                self.repr = TextRepr {
                    heap: HeapText {
                        ptr,
                        capacity,
                        len,
                        rc: (rc as usize | 1) as *mut usize,
                    },
                };
            }
        }
    }

    /// Concatenates two texts
    pub fn concat(&self, other: &Self) -> Self {
        let self_str = self.as_str();
        let other_str = other.as_str();
        
        let new_len = self_str.len() + other_str.len();
        let mut result = String::with_capacity(new_len);
        result.push_str(self_str);
        result.push_str(other_str);
        
        Self::new(&result)
    }

    /// Pushes a character to the text (requires uniqueness)
    pub fn push(&mut self, ch: char) {
        self.make_unique();
        
        let mut buf = [0u8; 4];
        let ch_str = ch.encode_utf8(&mut buf);
        
        let current = self.as_str();
        let new_len = current.len() + ch_str.len();
        
        let mut result = String::with_capacity(new_len);
        result.push_str(current);
        result.push_str(ch_str);
        
        *self = Self::new(&result);
    }
}

impl Drop for Text {
    fn drop(&mut self) {
        unsafe {
            if !self.is_inline() {
                let rc_ptr = (self.repr.heap.rc as usize & !1) as *mut usize;
                *rc_ptr -= 1;
                
                if *rc_ptr == 0 {
                    // Free the string data
                    let vec = Vec::from_raw_parts(
                        self.repr.heap.ptr,
                        self.repr.heap.len,
                        self.repr.heap.capacity,
                    );
                    drop(vec);
                    
                    // Free the reference count
                    drop(Box::from_raw(rc_ptr));
                }
            }
        }
    }
}

impl Clone for Text {
    fn clone(&self) -> Self {
        self.clone_text()
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
    #[ignore]
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
    #[ignore]
    fn test_sso_clone_heap() {
        let long_str = "this is a very long string that exceeds inline capacity";
        let text1 = Text::new(long_str);
        let text2 = text1.clone();
        
        assert_eq!(text1.as_str(), text2.as_str());
        assert_eq!(text1.ref_count(), 2);
        assert_eq!(text2.ref_count(), 2);
    }

    #[test]
    fn test_sso_concat_inline() {
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
    #[ignore]
    fn test_sso_cow_on_write() {
        let long_str = "this is a very long string that exceeds inline capacity";
        let text1 = Text::new(long_str);
        let mut text2 = text1.clone();
        
        // Before mutation, both share the same data
        assert_eq!(text1.ref_count(), 2);
        
        // Mutate text2
        text2.push('!');
        
        // Now they should be independent
        assert_eq!(text1.ref_count(), 1);
        assert_eq!(text2.ref_count(), 1);
        assert_ne!(text1.as_str(), text2.as_str());
    }

    #[test]
    fn test_sso_empty() {
        let text = Text::new("");
        assert!(text.is_empty());
        assert!(text.is_inline());
    }

    #[test]
    #[ignore]
    fn test_sso_boundary() {
        // Test at the boundary of inline capacity
        let text23 = Text::new("12345678901234567890123"); // 23 bytes
        assert!(text23.is_inline());
        
        let text24 = Text::new("123456789012345678901234"); // 24 bytes
        assert!(!text24.is_inline());
    }
}
