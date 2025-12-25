//! Native function wrappers for built-in JavaScript functions
//!
//! This module provides wrapper functions that convert JavaScript values
//! to native Rust types, call the appropriate built-in function, and
//! convert the result back to JavaScript values.

use crate::context::Context;
use crate::value::JSValue;
use crate::builtins::{math, console};

// ========== Math Functions ==========

/// Math.abs() wrapper
pub fn math_abs(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    // Get first argument or return NaN
    let arg = args.get(0).copied().unwrap_or(JSValue::undefined());

    // Convert to number
    let num = if let Some(i) = arg.to_int() {
        i as f64
    } else if let Some(f) = ctx.get_number(arg) {
        f
    } else {
        f64::NAN
    };

    // Calculate absolute value
    let result = math::abs(num);

    // Return as JSValue
    ctx.new_number(result)
        .map_err(|_| ctx.new_string("Out of memory").unwrap_or(JSValue::undefined()))
}

/// Math.floor() wrapper
pub fn math_floor(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let arg = args.get(0).copied().unwrap_or(JSValue::undefined());

    let num = if let Some(i) = arg.to_int() {
        i as f64
    } else if let Some(f) = ctx.get_number(arg) {
        f
    } else {
        f64::NAN
    };

    let result = math::floor(num);

    ctx.new_number(result)
        .map_err(|_| ctx.new_string("Out of memory").unwrap_or(JSValue::undefined()))
}

/// Math.ceil() wrapper
pub fn math_ceil(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let arg = args.get(0).copied().unwrap_or(JSValue::undefined());

    let num = if let Some(i) = arg.to_int() {
        i as f64
    } else if let Some(f) = ctx.get_number(arg) {
        f
    } else {
        f64::NAN
    };

    let result = math::ceil(num);

    ctx.new_number(result)
        .map_err(|_| ctx.new_string("Out of memory").unwrap_or(JSValue::undefined()))
}

/// Math.round() wrapper
pub fn math_round(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let arg = args.get(0).copied().unwrap_or(JSValue::undefined());

    let num = if let Some(i) = arg.to_int() {
        i as f64
    } else if let Some(f) = ctx.get_number(arg) {
        f
    } else {
        f64::NAN
    };

    let result = math::round(num);

    ctx.new_number(result)
        .map_err(|_| ctx.new_string("Out of memory").unwrap_or(JSValue::undefined()))
}

/// Math.min() wrapper
pub fn math_min(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    // Convert all arguments to f64
    let mut nums = alloc::vec::Vec::new();
    for arg in args {
        let num = if let Some(i) = arg.to_int() {
            i as f64
        } else if let Some(f) = ctx.get_number(*arg) {
            f
        } else {
            f64::NAN
        };
        nums.push(num);
    }

    let result = math::min(&nums);

    ctx.new_number(result)
        .map_err(|_| ctx.new_string("Out of memory").unwrap_or(JSValue::undefined()))
}

/// Math.max() wrapper
pub fn math_max(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    // Convert all arguments to f64
    let mut nums = alloc::vec::Vec::new();
    for arg in args {
        let num = if let Some(i) = arg.to_int() {
            i as f64
        } else if let Some(f) = ctx.get_number(*arg) {
            f
        } else {
            f64::NAN
        };
        nums.push(num);
    }

    let result = math::max(&nums);

    ctx.new_number(result)
        .map_err(|_| ctx.new_string("Out of memory").unwrap_or(JSValue::undefined()))
}

// ========== Console Functions ==========

/// console.log() wrapper
pub fn console_log_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    console::console_log(ctx, args);
    Ok(JSValue::undefined())
}

/// console.error() wrapper
pub fn console_error_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    console::console_error(ctx, args);
    Ok(JSValue::undefined())
}

/// console.warn() wrapper
pub fn console_warn_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    console::console_warn(ctx, args);
    Ok(JSValue::undefined())
}

/// console.info() wrapper
pub fn console_info_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    console::console_info(ctx, args);
    Ok(JSValue::undefined())
}
