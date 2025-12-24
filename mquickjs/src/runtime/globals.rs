//! Global functions and constants
//!
//! This module implements JavaScript global functions like parseInt, parseFloat,
//! isNaN, isFinite, and global constants like undefined, NaN, Infinity.

use crate::context::Context;
use crate::value::JSValue;
use alloc::string::String;
use core::str::FromStr;

/// parseInt() - Parses a string and returns an integer
///
/// # Arguments
///
/// * `ctx` - JavaScript execution context
/// * `string` - The string to parse
/// * `radix` - Optional radix (2-36), defaults to 10 (or 16 for "0x" prefix)
///
/// # Returns
///
/// Parsed integer or NaN if parsing fails
pub fn parse_int(ctx: &mut Context, string: JSValue, radix: Option<i32>) -> JSValue {
    // Get string representation
    let s = if let Some(str_ref) = ctx.get_string(string) {
        str_ref.trim_start()
    } else {
        // Try to convert to string first
        return JSValue::from_int(0); // Simplified: return 0 for non-strings
    };

    let radix = radix.unwrap_or(10).clamp(2, 36);

    // Handle empty string
    if s.is_empty() {
        return ctx.new_number(f64::NAN).unwrap_or(JSValue::undefined());
    }

    // Parse sign
    let (sign, s) = if s.starts_with('-') {
        (-1.0, &s[1..])
    } else if s.starts_with('+') {
        (1.0, &s[1..])
    } else {
        (1.0, s)
    };

    // Try to parse as i32 first for common case
    if radix == 10 {
        if let Ok(n) = s.parse::<i32>() {
            return JSValue::from_int((n as f64 * sign) as i32);
        }
    }

    // Simplified: parse what we can
    let mut result = 0i64;
    for c in s.chars() {
        if let Some(digit) = c.to_digit(radix as u32) {
            result = result * radix as i64 + digit as i64;
        } else {
            break; // Stop at first invalid character
        }
    }

    ctx.new_number(result as f64 * sign).unwrap_or(JSValue::undefined())
}

/// parseFloat() - Parses a string and returns a floating point number
///
/// # Arguments
///
/// * `ctx` - JavaScript execution context
/// * `string` - The string to parse
///
/// # Returns
///
/// Parsed number or NaN if parsing fails
pub fn parse_float(ctx: &mut Context, string: JSValue) -> JSValue {
    let s = if let Some(str_ref) = ctx.get_string(string) {
        str_ref.trim_start()
    } else {
        return ctx.new_number(f64::NAN).unwrap_or(JSValue::undefined());
    };

    match f64::from_str(s) {
        Ok(n) => ctx.new_number(n).unwrap_or(JSValue::undefined()),
        Err(_) => ctx.new_number(f64::NAN).unwrap_or(JSValue::undefined()),
    }
}

/// isNaN() - Determines whether a value is NaN
///
/// # Arguments
///
/// * `ctx` - JavaScript execution context
/// * `value` - The value to test
///
/// # Returns
///
/// true if the value is NaN, false otherwise
pub fn is_nan(ctx: &Context, value: JSValue) -> bool {
    if let Some(n) = ctx.get_number(value) {
        n.is_nan()
    } else {
        // Non-numbers convert to NaN
        true
    }
}

/// isFinite() - Determines whether a value is a finite number
///
/// # Arguments
///
/// * `ctx` - JavaScript execution context
/// * `value` - The value to test
///
/// # Returns
///
/// true if the value is finite, false otherwise
pub fn is_finite(ctx: &Context, value: JSValue) -> bool {
    if let Some(n) = ctx.get_number(value) {
        n.is_finite()
    } else {
        false
    }
}

/// encodeURI() - Encodes a URI by escaping certain characters
///
/// Simplified implementation that handles basic cases
pub fn encode_uri(_ctx: &mut Context, uri: JSValue) -> Result<JSValue, JSValue> {
    // TODO: Implement proper URI encoding
    // For now, just return the input unchanged
    Ok(uri)
}

/// decodeURI() - Decodes a URI by unescaping encoded characters
///
/// Simplified implementation that handles basic cases
pub fn decode_uri(_ctx: &mut Context, uri: JSValue) -> Result<JSValue, JSValue> {
    // TODO: Implement proper URI decoding
    // For now, just return the input unchanged
    Ok(uri)
}

/// encodeURIComponent() - Encodes a URI component
///
/// Simplified implementation
pub fn encode_uri_component(_ctx: &mut Context, component: JSValue) -> Result<JSValue, JSValue> {
    // TODO: Implement proper URI component encoding
    Ok(component)
}

/// decodeURIComponent() - Decodes a URI component
///
/// Simplified implementation
pub fn decode_uri_component(_ctx: &mut Context, component: JSValue) -> Result<JSValue, JSValue> {
    // TODO: Implement proper URI component decoding
    Ok(component)
}

/// eval() - Evaluates JavaScript code from a string
///
/// Placeholder implementation
pub fn eval(_ctx: &mut Context, _code: JSValue) -> Result<JSValue, JSValue> {
    // TODO: Implement eval using the compiler and VM
    // For now, return undefined
    Ok(JSValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_int() {
        let mut ctx = Context::new(4096);

        let str_val = ctx.new_string("42").unwrap();
        let result = parse_int(&mut ctx, str_val, None);
        assert_eq!(ctx.get_number(result), Some(42.0));

        let str_val = ctx.new_string("-100").unwrap();
        let result = parse_int(&mut ctx, str_val, None);
        assert_eq!(ctx.get_number(result), Some(-100.0));
    }

    #[test]
    fn test_parse_float() {
        let mut ctx = Context::new(4096);

        let str_val = ctx.new_string("3.14").unwrap();
        let result = parse_float(&mut ctx, str_val);
        assert_eq!(ctx.get_number(result), Some(3.14));

        let str_val = ctx.new_string("-2.5").unwrap();
        let result = parse_float(&mut ctx, str_val);
        assert_eq!(ctx.get_number(result), Some(-2.5));
    }

    #[test]
    fn test_is_nan() {
        let mut ctx = Context::new(4096);

        let nan_val = ctx.new_number(f64::NAN).unwrap();
        assert!(is_nan(&ctx, nan_val));

        let num_val = ctx.new_number(42.0).unwrap();
        assert!(!is_nan(&ctx, num_val));
    }

    #[test]
    fn test_is_finite() {
        let mut ctx = Context::new(4096);

        let finite_val = ctx.new_number(42.0).unwrap();
        assert!(is_finite(&ctx, finite_val));

        let inf_val = ctx.new_number(f64::INFINITY).unwrap();
        assert!(!is_finite(&ctx, inf_val));

        let nan_val = ctx.new_number(f64::NAN).unwrap();
        assert!(!is_finite(&ctx, nan_val));
    }
}
