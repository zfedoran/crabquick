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
    use crate::runtime::conversion::to_string;

    // Convert to string first
    let s = to_string(ctx, string);
    let s = s.trim_start();

    // Handle empty string after trimming
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

    // Determine radix
    let mut actual_radix = radix.unwrap_or(0);
    let mut s = s;

    // Handle "0x" or "0X" prefix for hex
    if actual_radix == 0 || actual_radix == 16 {
        if s.len() >= 2 && (s.starts_with("0x") || s.starts_with("0X")) {
            actual_radix = 16;
            s = &s[2..];
        }
    }

    // Default to radix 10 if not specified
    if actual_radix == 0 {
        actual_radix = 10;
    }

    // Validate radix range
    if actual_radix < 2 || actual_radix > 36 {
        return ctx.new_number(f64::NAN).unwrap_or(JSValue::undefined());
    }

    // Parse digits until we hit an invalid character
    let mut result = 0i64;
    let mut parsed_any = false;

    for c in s.chars() {
        if let Some(digit) = c.to_digit(actual_radix as u32) {
            result = result.saturating_mul(actual_radix as i64).saturating_add(digit as i64);
            parsed_any = true;
        } else {
            break; // Stop at first invalid character (JavaScript behavior)
        }
    }

    // If no valid digits were parsed, return NaN
    if !parsed_any {
        return ctx.new_number(f64::NAN).unwrap_or(JSValue::undefined());
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
    use crate::runtime::conversion::to_string;

    // Convert to string first
    let s = to_string(ctx, string);
    let s = s.trim_start();

    // Handle empty string
    if s.is_empty() {
        return ctx.new_number(f64::NAN).unwrap_or(JSValue::undefined());
    }

    // Try to parse as much as possible (JavaScript parseFloat is lenient)
    // It parses until it hits an invalid character
    let mut end_idx = 0;
    let mut has_dot = false;
    let mut has_e = false;
    let mut chars = s.chars().peekable();

    // Handle sign
    if let Some(&c) = chars.peek() {
        if c == '+' || c == '-' {
            end_idx += 1;
            chars.next();
        }
    }

    // Parse number
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            end_idx += 1;
            chars.next();
        } else if c == '.' && !has_dot && !has_e {
            has_dot = true;
            end_idx += 1;
            chars.next();
        } else if (c == 'e' || c == 'E') && !has_e {
            has_e = true;
            end_idx += 1;
            chars.next();
            // Handle sign after 'e'
            if let Some(&next_c) = chars.peek() {
                if next_c == '+' || next_c == '-' {
                    end_idx += 1;
                    chars.next();
                }
            }
        } else {
            break;
        }
    }

    // Parse the valid portion
    let parse_str = &s[..end_idx];
    if parse_str.is_empty() || parse_str == "+" || parse_str == "-" {
        return ctx.new_number(f64::NAN).unwrap_or(JSValue::undefined());
    }

    match f64::from_str(parse_str) {
        Ok(n) => ctx.new_number(n).unwrap_or(JSValue::undefined()),
        Err(_) => {
            // Handle special JavaScript values
            if s.starts_with("Infinity") || s.starts_with("+Infinity") {
                ctx.new_number(f64::INFINITY).unwrap_or(JSValue::undefined())
            } else if s.starts_with("-Infinity") {
                ctx.new_number(f64::NEG_INFINITY).unwrap_or(JSValue::undefined())
            } else {
                ctx.new_number(f64::NAN).unwrap_or(JSValue::undefined())
            }
        }
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
    use crate::runtime::conversion::to_number;

    // JavaScript isNaN applies ToNumber conversion first
    let num = to_number(ctx, value);
    num.is_nan()
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
    use crate::runtime::conversion::to_number;

    // JavaScript isFinite applies ToNumber conversion first
    let num = to_number(ctx, value);
    num.is_finite()
}

/// encodeURI() - Encodes a URI by escaping certain characters
///
/// Encodes all characters except: A-Z a-z 0-9 ; , / ? : @ & = + $ - _ . ! ~ * ' ( ) #
pub fn encode_uri(ctx: &mut Context, uri: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_string;
    use alloc::string::String;

    let s = to_string(ctx, uri);
    let mut result = String::new();

    for c in s.chars() {
        if is_uri_reserved(c) || is_uri_unreserved(c) || c == '#' {
            result.push(c);
        } else {
            // Percent-encode the character
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            for byte in encoded.bytes() {
                result.push('%');
                result.push_str(&alloc::format!("{:02X}", byte));
            }
        }
    }

    ctx.new_string(&result)
        .map_err(|_| ctx.new_string("Out of memory").unwrap_or(JSValue::undefined()))
}

/// decodeURI() - Decodes a URI by unescaping encoded characters
///
/// Decodes percent-encoded sequences except for reserved characters
pub fn decode_uri(ctx: &mut Context, uri: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_string;
    use alloc::string::String;
    use alloc::vec::Vec;

    let s = to_string(ctx, uri);
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            // Try to decode the percent-encoded sequence
            let hex = alloc::format!("{}{}", chars[i + 1], chars[i + 2]);
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                // Check if this is part of a multi-byte UTF-8 sequence
                let decoded_char = char::from_u32(byte as u32);
                if let Some(c) = decoded_char {
                    // Don't decode reserved characters in decodeURI
                    if is_uri_reserved(c) || c == '#' {
                        result.push('%');
                        result.push(chars[i + 1]);
                        result.push(chars[i + 2]);
                    } else {
                        result.push(c);
                    }
                } else {
                    // Invalid sequence, keep as-is
                    result.push('%');
                    result.push(chars[i + 1]);
                    result.push(chars[i + 2]);
                }
                i += 3;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    ctx.new_string(&result)
        .map_err(|_| ctx.new_string("URI malformed").unwrap_or(JSValue::undefined()))
}

/// encodeURIComponent() - Encodes a URI component
///
/// Encodes all characters except: A-Z a-z 0-9 - _ . ! ~ * ' ( )
pub fn encode_uri_component(ctx: &mut Context, component: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_string;
    use alloc::string::String;

    let s = to_string(ctx, component);
    let mut result = String::new();

    for c in s.chars() {
        if is_uri_unreserved(c) {
            result.push(c);
        } else {
            // Percent-encode the character
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            for byte in encoded.bytes() {
                result.push('%');
                result.push_str(&alloc::format!("{:02X}", byte));
            }
        }
    }

    ctx.new_string(&result)
        .map_err(|_| ctx.new_string("Out of memory").unwrap_or(JSValue::undefined()))
}

/// decodeURIComponent() - Decodes a URI component
///
/// Decodes all percent-encoded sequences
pub fn decode_uri_component(ctx: &mut Context, component: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_string;
    use alloc::string::String;
    use alloc::vec::Vec;

    let s = to_string(ctx, component);
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            // Try to decode the percent-encoded sequence
            let hex = alloc::format!("{}{}", chars[i + 1], chars[i + 2]);
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                // Simple single-byte decoding (UTF-8 multi-byte sequences would need more work)
                let decoded_char = char::from_u32(byte as u32);
                if let Some(c) = decoded_char {
                    result.push(c);
                } else {
                    // Invalid sequence, keep as-is
                    result.push('%');
                    result.push(chars[i + 1]);
                    result.push(chars[i + 2]);
                }
                i += 3;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    ctx.new_string(&result)
        .map_err(|_| ctx.new_string("URI malformed").unwrap_or(JSValue::undefined()))
}

/// Check if a character is a URI reserved character
fn is_uri_reserved(c: char) -> bool {
    matches!(c, ';' | ',' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$')
}

/// Check if a character is a URI unreserved character
fn is_uri_unreserved(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')')
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
