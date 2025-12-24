//! Runtime support (type conversions, operators, global functions)

pub mod conversion;
pub mod operators;
pub mod compare;
pub mod globals;

// Re-exports
pub use conversion::{to_number, to_int32, to_string, to_boolean};
pub use operators::{add, subtract, multiply, divide};
pub use compare::{strict_equal, abstract_equal, less_than};
pub use globals::{parse_int, parse_float, is_nan, is_finite};

use crate::context::Context;
use crate::value::JSValue;

/// Initialize the JavaScript runtime environment
///
/// This function sets up the global object with all built-in constructors,
/// prototypes, and global functions.
///
/// # Arguments
///
/// * `ctx` - JavaScript execution context
///
/// # Returns
///
/// The global object with all built-ins installed
pub fn init_runtime(_ctx: &mut Context) -> Result<JSValue, JSValue> {
    // TODO: Create global object
    // TODO: Install Object constructor and prototype
    // TODO: Install Array constructor and prototype
    // TODO: Install String constructor and prototype
    // TODO: Install Number constructor and prototype
    // TODO: Install Boolean constructor and prototype
    // TODO: Install Function constructor and prototype
    // TODO: Install Math object
    // TODO: Install Error constructors
    // TODO: Install Console object
    // TODO: Install global functions (parseInt, parseFloat, etc.)
    // TODO: Install global constants (undefined, NaN, Infinity)

    // For now, just return undefined
    Ok(JSValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_runtime() {
        let mut ctx = Context::new(8192);
        let result = init_runtime(&mut ctx);
        assert!(result.is_ok());
    }
}
