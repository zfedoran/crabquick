//! Comparison operations

use crate::value::JSValue;

/// Strict equality (===)
pub fn strict_equal(_left: JSValue, _right: JSValue) -> bool {
    // TODO: Implement strict equality without type coercion
    false
}

/// Abstract equality (==)
pub fn abstract_equal(_left: JSValue, _right: JSValue) -> bool {
    // TODO: Implement abstract equality with type coercion
    false
}

/// Less than operator
pub fn less_than(_left: JSValue, _right: JSValue) -> bool {
    // TODO: Implement less than comparison
    false
}
