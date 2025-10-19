//! Reference-counted smart pointer for Kiv runtime.
//!
//! This is a simplified version of `std::rc::Rc` for demonstration purposes.

use std::ops::Deref;
use std::ptr::NonNull;

/// A reference-counted pointer to a value of type `T`.
pub struct Rc<T> {
    ptr: NonNull<RcBox<T>>,
}

struct RcBox<T> {
    strong: usize,
    value: T,
}

impl<T> Rc<T> {
    /// Creates a new reference-counted pointer.
    pub fn new(value: T) -> Self {
        let boxed = Box::new(RcBox { strong: 1, value });
        Self {
            ptr: unsafe { NonNull::new_unchecked(Box::into_raw(boxed)) },
        }
    }

    /// Gets the number of strong references to this value.
    pub fn strong_count(&self) -> usize {
        unsafe { self.ptr.as_ref().strong }
    }

    /// Returns a raw pointer to the inner value.
    pub fn as_ptr(&self) -> *const T {
        unsafe { &self.ptr.as_ref().value as *const T }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        unsafe {
            let inner = self.ptr.as_ptr();
            (*inner).strong += 1;
            Self { ptr: self.ptr }
        }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.ptr.as_ptr();
            (*inner).strong -= 1;

            if (*inner).strong == 0 {
                // Last reference - deallocate
                let _ = Box::from_raw(inner);
            }
        }
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.ptr.as_ref().value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc_basic() {
        let rc = Rc::new(42);
        assert_eq!(*rc, 42);
        assert_eq!(rc.strong_count(), 1);
    }

    #[test]
    fn test_rc_clone() {
        let rc1 = Rc::new(100);
        assert_eq!(rc1.strong_count(), 1);

        let rc2 = rc1.clone();
        assert_eq!(rc1.strong_count(), 2);
        assert_eq!(rc2.strong_count(), 2);
        assert_eq!(*rc1, *rc2);
    }

    #[test]
    fn test_rc_drop() {
        let rc1 = Rc::new(200);
        let rc2 = rc1.clone();
        assert_eq!(rc1.strong_count(), 2);

        drop(rc2);
        assert_eq!(rc1.strong_count(), 1);
    }
}
