//! JavaScript string implementation
//!
//! This module re-exports the main JSString implementation from value::string.
//! The actual UTF-8 storage, hash caching, and string operations are implemented
//! in crate::value::JSString.
//!
//! ## Features Implemented:
//! - Proper UTF-8 storage and handling
//! - Length computation (both byte and character count)
//! - Hash caching for faster property lookups
//! - ASCII and numeric flags for optimizations
//! - String interning support via AtomTable

// Re-export the main JSString implementation from value module
pub use crate::value::{JSString, JSStringHeader};

/// Helper to create strings from context
///
/// This is a convenience wrapper around Context::new_string().
/// Strings should always be allocated via the Context to ensure
/// proper memory management and GC integration.
///
/// Example:
/// ```rust,ignore
/// let str_value = ctx.new_string("hello")?;
/// ```
pub struct JSStringBuilder;

impl JSStringBuilder {
    /// Creates a new string through the context
    ///
    /// This is the recommended way to create strings in CrabQuick.
    /// The string will be properly allocated in the arena with GC support.
    #[inline]
    pub fn new(ctx: &mut crate::context::Context, s: &str) -> Result<crate::value::JSValue, crate::memory::allocator::OutOfMemory> {
        ctx.new_string(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;

    #[test]
    fn test_string_creation() {
        let mut ctx = Context::new(8192);
        let result = JSStringBuilder::new(&mut ctx, "hello");
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_retrieval() {
        let mut ctx = Context::new(8192);
        let str_val = JSStringBuilder::new(&mut ctx, "hello").unwrap();
        let retrieved = ctx.get_string(str_val);
        assert_eq!(retrieved, Some("hello"));
    }

    #[test]
    fn test_utf8_string() {
        let mut ctx = Context::new(8192);
        let str_val = JSStringBuilder::new(&mut ctx, "café ☕").unwrap();
        let retrieved = ctx.get_string(str_val);
        assert_eq!(retrieved, Some("café ☕"));
    }

    #[test]
    fn test_empty_string() {
        let mut ctx = Context::new(8192);
        let str_val = JSStringBuilder::new(&mut ctx, "").unwrap();
        let retrieved = ctx.get_string(str_val);
        assert_eq!(retrieved, Some(""));
    }

    #[test]
    fn test_numeric_string() {
        let mut ctx = Context::new(8192);
        let str_val = JSStringBuilder::new(&mut ctx, "123.45").unwrap();
        let retrieved = ctx.get_string(str_val);
        assert_eq!(retrieved, Some("123.45"));
    }

    #[test]
    fn test_ascii_string() {
        let mut ctx = Context::new(8192);
        let str_val = JSStringBuilder::new(&mut ctx, "hello world").unwrap();
        let retrieved = ctx.get_string(str_val);
        assert_eq!(retrieved, Some("hello world"));
    }
}
