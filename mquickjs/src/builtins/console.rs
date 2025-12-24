//! Console built-in object
//!
//! Implements console.log, console.error, console.warn, console.info

use crate::context::Context;
use crate::value::JSValue;
use alloc::string::String;
use alloc::vec::Vec;

/// Format and print values to stdout (or designated output)
///
/// This is a helper function used by all console methods
fn format_values(ctx: &Context, args: &[JSValue]) -> String {
    let mut result = String::new();

    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }

        // Convert value to string representation
        let s = value_to_display_string(ctx, *arg);
        result.push_str(&s);
    }

    result
}

/// Convert a JSValue to a display string
fn value_to_display_string(ctx: &Context, value: JSValue) -> String {
    if value.is_null() {
        return String::from("null");
    }

    if value.is_undefined() {
        return String::from("undefined");
    }

    if let Some(b) = value.to_bool() {
        return if b { String::from("true") } else { String::from("false") };
    }

    if let Some(n) = ctx.get_number(value) {
        // Format number
        if n.is_nan() {
            return String::from("NaN");
        }
        if n.is_infinite() {
            return if n > 0.0 {
                String::from("Infinity")
            } else {
                String::from("-Infinity")
            };
        }
        // Use alloc::format! for no_std compatibility
        return alloc::format!("{}", n);
    }

    if let Some(s) = ctx.get_string(value) {
        return String::from(s);
    }

    // Object
    if value.is_object() {
        return String::from("[object Object]");
    }

    String::from("undefined")
}

/// console.log() - Logs messages to the console
///
/// # Arguments
///
/// * `ctx` - JavaScript execution context
/// * `args` - Arguments to log
pub fn console_log(ctx: &Context, args: &[JSValue]) {
    let message = format_values(ctx, args);

    #[cfg(not(test))]
    {
        // In no_std environment, we can't use println! directly
        // This is a placeholder - actual implementation would need
        // to use a platform-specific output method
        // For now, we'll just do nothing in no_std mode
        let _ = message;
    }

    #[cfg(test)]
    {
        println!("{}", message);
    }
}

/// console.error() - Logs error messages to the console
///
/// # Arguments
///
/// * `ctx` - JavaScript execution context
/// * `args` - Arguments to log as error
pub fn console_error(ctx: &Context, args: &[JSValue]) {
    let message = format_values(ctx, args);

    #[cfg(not(test))]
    {
        let _ = message;
    }

    #[cfg(test)]
    {
        eprintln!("{}", message);
    }
}

/// console.warn() - Logs warning messages to the console
///
/// # Arguments
///
/// * `ctx` - JavaScript execution context
/// * `args` - Arguments to log as warning
pub fn console_warn(ctx: &Context, args: &[JSValue]) {
    let message = format_values(ctx, args);

    #[cfg(not(test))]
    {
        let _ = message;
    }

    #[cfg(test)]
    {
        eprintln!("Warning: {}", message);
    }
}

/// console.info() - Logs informational messages to the console
///
/// # Arguments
///
/// * `ctx` - JavaScript execution context
/// * `args` - Arguments to log as info
pub fn console_info(ctx: &Context, args: &[JSValue]) {
    // console.info is typically the same as console.log
    console_log(ctx, args);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_log() {
        let mut ctx = Context::new(4096);

        let msg = ctx.new_string("Hello, world!").unwrap();
        console_log(&ctx, &[msg]);
        // Output is printed, can't easily test without capturing stdout
    }

    #[test]
    fn test_format_values() {
        let mut ctx = Context::new(4096);

        let str_val = ctx.new_string("hello").unwrap();
        let num_val = ctx.new_number(42.0).unwrap();
        let bool_val = JSValue::bool(true);

        let result = format_values(&ctx, &[str_val, num_val, bool_val]);
        assert_eq!(result, "hello 42 true");
    }

    #[test]
    fn test_value_to_display_string() {
        let mut ctx = Context::new(4096);

        assert_eq!(value_to_display_string(&ctx, JSValue::null()), "null");
        assert_eq!(value_to_display_string(&ctx, JSValue::undefined()), "undefined");
        assert_eq!(value_to_display_string(&ctx, JSValue::bool(true)), "true");
        assert_eq!(value_to_display_string(&ctx, JSValue::bool(false)), "false");

        let num = ctx.new_number(3.14).unwrap();
        assert_eq!(value_to_display_string(&ctx, num), "3.14");

        let str_val = ctx.new_string("test").unwrap();
        assert_eq!(value_to_display_string(&ctx, str_val), "test");
    }
}
