//! Utility functions

pub mod dtoa;
pub mod strtod;
pub mod utf8;
pub mod bitpack;

// Re-exports
pub use dtoa::format_number;
pub use strtod::parse_number;
pub use utf8::{is_utf8_char_boundary, count_utf8_chars};
