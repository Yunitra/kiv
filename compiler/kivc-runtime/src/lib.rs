//! Runtime library for the Kiv programming language.
//!
//! This crate provides the runtime support needed by compiled Kiv programs,
//! including memory management (reference counting), the Text type (CoW strings),
//! and built-in functions.
//!
//! The runtime is designed to be linked with generated code and provides
//! C-compatible ABI for interoperability.

mod builtins;
mod rc;
mod sso;
mod text;

pub use builtins::*;
pub use rc::Rc;
pub use sso::Text as SsoText;
pub use text::Text;
