//! Built-in functions for the Kiv runtime.
//!
//! These functions provide core functionality that is available to all Kiv programs,
//! such as printing to stdout, type conversions, and basic I/O.

use crate::text::Text;

/// Prints a Text value to stdout (C ABI).
///
/// # Safety
/// The pointer must be a valid Text pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kiv_print_text(text: *const Text) {
    unsafe {
        if text.is_null() {
            return;
        }

        let text_ref = &*text;
        if let Some(s) = text_ref.as_str() {
            println!("{}", s);
        }
    }
}

/// Prints an integer to stdout (C ABI).
#[unsafe(no_mangle)]
pub extern "C" fn kiv_print_int(value: i64) {
    println!("{}", value);
}

/// Prints a float to stdout (C ABI).
#[unsafe(no_mangle)]
pub extern "C" fn kiv_print_float(value: f64) {
    println!("{}", value);
}

/// Prints a boolean to stdout (C ABI).
#[unsafe(no_mangle)]
pub extern "C" fn kiv_print_bool(value: bool) {
    println!("{}", value);
}

/// Converts an integer to a Text value (C ABI).
#[unsafe(no_mangle)]
pub extern "C" fn kiv_int_to_text(value: i64) -> *mut Text {
    let s = value.to_string();
    let text = Text::from_str(&s);
    Box::into_raw(Box::new(text))
}

/// Converts a float to a Text value (C ABI).
#[unsafe(no_mangle)]
pub extern "C" fn kiv_float_to_text(value: f64) -> *mut Text {
    let s = value.to_string();
    let text = Text::from_str(&s);
    Box::into_raw(Box::new(text))
}

/// Converts a boolean to a Text value (C ABI).
#[unsafe(no_mangle)]
pub extern "C" fn kiv_bool_to_text(value: bool) -> *mut Text {
    let s = if value { "true" } else { "false" };
    let text = Text::from_str(s);
    Box::into_raw(Box::new(text))
}

/// Runtime panic handler (C ABI).
///
/// # Safety
/// The pointer must be a valid Text pointer describing the panic message.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kiv_panic(message: *const Text) -> ! {
    unsafe {
        if !message.is_null() {
            let text_ref = &*message;
            if let Some(s) = text_ref.as_str() {
                panic!("Kiv runtime panic: {}", s);
            }
        }

        panic!("Kiv runtime panic");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_to_text() {
        let text_ptr = kiv_int_to_text(42);
        unsafe {
            let text = &*text_ptr;
            assert_eq!(text.as_str(), Some("42"));
            let _ = Box::from_raw(text_ptr); // Clean up
        }
    }

    #[test]
    fn test_float_to_text() {
        let text_ptr = kiv_float_to_text(3.14);
        unsafe {
            let text = &*text_ptr;
            assert!(text.as_str().unwrap().starts_with("3.14"));
            let _ = Box::from_raw(text_ptr); // Clean up
        }
    }

    #[test]
    fn test_bool_to_text() {
        let true_ptr = kiv_bool_to_text(true);
        let false_ptr = kiv_bool_to_text(false);

        unsafe {
            assert_eq!((*true_ptr).as_str(), Some("true"));
            assert_eq!((*false_ptr).as_str(), Some("false"));

            let _ = Box::from_raw(true_ptr);
            let _ = Box::from_raw(false_ptr);
        }
    }
}
