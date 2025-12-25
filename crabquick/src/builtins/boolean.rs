//! Boolean built-in constructor and methods

use crate::context::Context;
use crate::value::JSValue;

/// Boolean() constructor
///
/// Converts a value to a boolean
pub fn boolean_constructor(ctx: &Context, value: Option<JSValue>) -> JSValue {
    match value {
        None => JSValue::bool(false),
        Some(val) => {
            // Convert to boolean using JavaScript truthiness rules
            let b = to_boolean(ctx, val);
            JSValue::bool(b)
        }
    }
}

/// Converts a value to boolean using JavaScript truthiness
pub fn to_boolean(ctx: &Context, value: JSValue) -> bool {
    if value.is_null() || value.is_undefined() {
        return false;
    }

    if let Some(b) = value.to_bool() {
        return b;
    }

    if let Some(n) = ctx.get_number(value) {
        return n != 0.0 && !n.is_nan();
    }

    if let Some(s) = ctx.get_string(value) {
        return !s.is_empty();
    }

    // Objects are truthy
    true
}

/// Boolean.prototype.toString() - Returns string representation
pub fn to_string(ctx: &mut Context, bool_val: JSValue) -> Result<JSValue, JSValue> {
    if let Some(b) = bool_val.to_bool() {
        let s = if b { "true" } else { "false" };
        ctx.new_string(s).map_err(|_| JSValue::exception())
    } else {
        Err(JSValue::exception())
    }
}

/// Boolean.prototype.valueOf() - Returns primitive boolean value
pub fn value_of(bool_val: JSValue) -> Option<bool> {
    bool_val.to_bool()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_constructor() {
        let ctx = Context::new(4096);

        let b = boolean_constructor(&ctx, Some(JSValue::from_int(1)));
        assert_eq!(b.to_bool(), Some(true));

        let b = boolean_constructor(&ctx, Some(JSValue::from_int(0)));
        assert_eq!(b.to_bool(), Some(false));

        let b = boolean_constructor(&ctx, None);
        assert_eq!(b.to_bool(), Some(false));
    }

    #[test]
    fn test_to_boolean() {
        let mut ctx = Context::new(4096);

        assert!(!to_boolean(&ctx, JSValue::null()));
        assert!(!to_boolean(&ctx, JSValue::undefined()));
        assert!(to_boolean(&ctx, JSValue::bool(true)));
        assert!(!to_boolean(&ctx, JSValue::bool(false)));

        let zero = ctx.new_number(0.0).unwrap();
        assert!(!to_boolean(&ctx, zero));

        let one = ctx.new_number(1.0).unwrap();
        assert!(to_boolean(&ctx, one));

        let empty = ctx.new_string("").unwrap();
        assert!(!to_boolean(&ctx, empty));

        let non_empty = ctx.new_string("hello").unwrap();
        assert!(to_boolean(&ctx, non_empty));
    }

    #[test]
    fn test_to_string() {
        let mut ctx = Context::new(4096);

        let result = to_string(&mut ctx, JSValue::bool(true)).unwrap();
        assert_eq!(ctx.get_string(result).unwrap(), "true");

        let result = to_string(&mut ctx, JSValue::bool(false)).unwrap();
        assert_eq!(ctx.get_string(result).unwrap(), "false");
    }

    #[test]
    fn test_value_of() {
        assert_eq!(value_of(JSValue::bool(true)), Some(true));
        assert_eq!(value_of(JSValue::bool(false)), Some(false));
        assert_eq!(value_of(JSValue::null()), None);
    }
}
