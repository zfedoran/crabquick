//! Type conversion functions

use crate::value::JSValue;
use crate::context::Context;

/// Converts a value to a number (ES5 9.3 ToNumber)
///
/// # Rules
/// - undefined → NaN
/// - null → 0
/// - true → 1, false → 0
/// - Number → return as-is
/// - String → parse as number (empty string → 0, "123" → 123, "abc" → NaN)
/// - Object → convert to primitive first (not implemented yet)
pub fn to_number(ctx: &Context, value: JSValue) -> f64 {
    // undefined → NaN
    if value.is_undefined() {
        return f64::NAN;
    }

    // null → 0
    if value.is_null() {
        return 0.0;
    }

    // Boolean → 1 or 0
    if let Some(b) = value.to_bool() {
        return if b { 1.0 } else { 0.0 };
    }

    // Number (integer)
    if let Some(i) = value.to_int() {
        return i as f64;
    }

    // Number (boxed float64)
    if let Some(f) = ctx.get_number(value) {
        return f;
    }

    // String
    if let Some(s) = ctx.get_string(value) {
        return string_to_number(s);
    }

    // Object or other types → NaN (ToPrimitive not implemented yet)
    f64::NAN
}

/// Converts a string to a number following JavaScript rules
fn string_to_number(s: &str) -> f64 {
    // Trim whitespace
    let s = s.trim();

    // Empty string → 0
    if s.is_empty() {
        return 0.0;
    }

    // Try to parse as f64
    match s.parse::<f64>() {
        Ok(n) => n,
        Err(_) => {
            // Special cases that Rust's parse doesn't handle
            match s {
                "Infinity" | "+Infinity" => f64::INFINITY,
                "-Infinity" => f64::NEG_INFINITY,
                _ => f64::NAN,
            }
        }
    }
}

/// Converts a value to a 32-bit signed integer (ES5 9.5 ToInt32)
///
/// Converts to number first, then applies modulo 2^32 and maps to signed range
pub fn to_int32(ctx: &Context, value: JSValue) -> i32 {
    let number = to_number(ctx, value);

    // NaN, Infinity, -Infinity → 0
    if !number.is_finite() {
        return 0;
    }

    // Apply modulo 2^32 and convert to signed
    let int32bit = (number as i64) as i32;
    int32bit
}

/// Converts a value to a string (ES5 9.8 ToString)
///
/// # Rules
/// - undefined → "undefined"
/// - null → "null"
/// - true → "true", false → "false"
/// - Number → format as string
/// - String → return as-is
/// - Object → call toString() (not implemented yet)
pub fn to_string(ctx: &Context, value: JSValue) -> alloc::string::String {
    use alloc::string::ToString;
    use alloc::format;

    // undefined → "undefined"
    if value.is_undefined() {
        return alloc::string::String::from("undefined");
    }

    // null → "null"
    if value.is_null() {
        return alloc::string::String::from("null");
    }

    // Boolean → "true" or "false"
    if let Some(b) = value.to_bool() {
        return if b {
            alloc::string::String::from("true")
        } else {
            alloc::string::String::from("false")
        };
    }

    // Number (integer)
    if let Some(i) = value.to_int() {
        return format!("{}", i);
    }

    // Number (boxed float64)
    if let Some(f) = ctx.get_number(value) {
        return number_to_string(f);
    }

    // String
    if let Some(s) = ctx.get_string(value) {
        return alloc::string::String::from(s);
    }

    // Object or other types → "[object Object]" (ToPrimitive not implemented yet)
    alloc::string::String::from("[object Object]")
}

/// Converts a number to a string following JavaScript rules
fn number_to_string(n: f64) -> alloc::string::String {
    use alloc::string::ToString;
    use alloc::format;

    if n.is_nan() {
        return alloc::string::String::from("NaN");
    }

    if n.is_infinite() {
        return if n > 0.0 {
            alloc::string::String::from("Infinity")
        } else {
            alloc::string::String::from("-Infinity")
        };
    }

    if n == 0.0 {
        return alloc::string::String::from("0");
    }

    // For now, use simple formatting
    // A full implementation would need to handle exponential notation properly
    format!("{}", n)
}

/// Converts a value to a boolean (ES5 9.2 ToBoolean)
///
/// # Rules
/// - undefined, null → false
/// - false → false, true → true
/// - Number: 0, NaN → false, else → true
/// - String: "" → false, else → true
/// - Object → true
pub fn to_boolean(ctx: &Context, value: JSValue) -> bool {
    // undefined, null → false
    if value.is_undefined() || value.is_null() {
        return false;
    }

    // Boolean → return as-is
    if let Some(b) = value.to_bool() {
        return b;
    }

    // Number (integer): 0 → false, else → true
    if let Some(i) = value.to_int() {
        return i != 0;
    }

    // Number (boxed float64): 0, NaN → false, else → true
    if let Some(f) = ctx.get_number(value) {
        return f != 0.0 && !f.is_nan();
    }

    // String: "" → false, else → true
    if let Some(s) = ctx.get_string(value) {
        return !s.is_empty();
    }

    // Object or other pointer types → true
    if value.is_ptr() {
        return true;
    }

    // Default to false for unknown types
    false
}
