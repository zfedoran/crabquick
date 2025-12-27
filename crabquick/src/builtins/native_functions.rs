//! Native function wrappers for built-in JavaScript functions
//!
//! This module provides wrapper functions that convert JavaScript values
//! to native Rust types, call the appropriate built-in function, and
//! convert the result back to JavaScript values.

use crate::context::Context;
use crate::value::JSValue;
use crate::builtins::{math, console, array, string, object};

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

/// Math.pow() wrapper
pub fn math_pow(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let base = if args.is_empty() {
        f64::NAN
    } else if let Some(i) = args[0].to_int() {
        i as f64
    } else if let Some(f) = ctx.get_number(args[0]) {
        f
    } else {
        f64::NAN
    };

    let exp = if args.len() < 2 {
        f64::NAN
    } else if let Some(i) = args[1].to_int() {
        i as f64
    } else if let Some(f) = ctx.get_number(args[1]) {
        f
    } else {
        f64::NAN
    };

    let result = libm::pow(base, exp);

    ctx.new_number(result)
        .map_err(|_| ctx.new_string("Out of memory").unwrap_or(JSValue::undefined()))
}

/// Math.sqrt() wrapper
pub fn math_sqrt(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let num = if args.is_empty() {
        f64::NAN
    } else if let Some(i) = args[0].to_int() {
        i as f64
    } else if let Some(f) = ctx.get_number(args[0]) {
        f
    } else {
        f64::NAN
    };

    let result = libm::sqrt(num);

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

// ========== Array.prototype Methods ==========

/// Array.prototype.push() wrapper
pub fn array_push_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let len = array::array_push(ctx, this, args)?;
    ctx.new_number(len as f64)
        .map_err(|_| JSValue::exception())
}

/// Array.prototype.pop() wrapper
pub fn array_pop_native(ctx: &mut Context, this: JSValue, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    array::array_pop(ctx, this)
}

/// Array.prototype.shift() wrapper
pub fn array_shift_native(ctx: &mut Context, this: JSValue, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    array::array_shift(ctx, this)
}

/// Array.prototype.unshift() wrapper
pub fn array_unshift_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let len = array::array_unshift(ctx, this, args)?;
    ctx.new_number(len as f64)
        .map_err(|_| JSValue::exception())
}

/// Array.prototype.slice() wrapper
pub fn array_slice_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let start = args.get(0).map(|v| to_int32(ctx, *v));
    let end = args.get(1).map(|v| to_int32(ctx, *v));

    array::array_slice(ctx, this, start, end)
}

/// Array.prototype.concat() wrapper
pub fn array_concat_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    array::array_concat(ctx, this, args)
}

/// Array.prototype.indexOf() wrapper
pub fn array_index_of_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let search_element = args.get(0).copied().unwrap_or(JSValue::undefined());
    let from_index = args.get(1).map(|v| to_int32(ctx, *v));

    let index = array::array_index_of(ctx, this, search_element, from_index)?;
    ctx.new_number(index as f64)
        .map_err(|_| JSValue::exception())
}

/// Array.prototype.includes() wrapper
pub fn array_includes_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let search_element = args.get(0).copied().unwrap_or(JSValue::undefined());
    let from_index = args.get(1).map(|v| to_int32(ctx, *v));

    let result = array::array_includes(ctx, this, search_element, from_index)?;
    Ok(JSValue::bool(result))
}

/// Array.prototype.splice() wrapper
pub fn array_splice_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let start = args.get(0).map(|v| to_int32(ctx, *v)).unwrap_or(0);
    let delete_count = args.get(1).map(|v| to_int32(ctx, *v));
    let items = if args.len() > 2 { &args[2..] } else { &[] };

    array::array_splice(ctx, this, start, delete_count, items)
}

// ========== String.prototype Methods ==========

/// String.prototype.charAt() wrapper
pub fn string_char_at_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let index = args.get(0).map(|v| to_int32(ctx, *v)).unwrap_or(0);
    string::char_at(ctx, this, index)
}

/// String.prototype.charCodeAt() wrapper
pub fn string_char_code_at_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let index = args.get(0).map(|v| to_int32(ctx, *v)).unwrap_or(0);
    let code = string::char_code_at(ctx, this, index)?;
    ctx.new_number(code as f64)
        .map_err(|_| JSValue::exception())
}

/// String.prototype.slice() wrapper
pub fn string_slice_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let start = args.get(0).map(|v| to_int32(ctx, *v)).unwrap_or(0);
    let end = args.get(1).map(|v| to_int32(ctx, *v));

    string::slice(ctx, this, start, end)
}

/// String.prototype.substring() wrapper
pub fn string_substring_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let start = args.get(0).map(|v| to_int32(ctx, *v)).unwrap_or(0);
    let end = args.get(1).map(|v| to_int32(ctx, *v));

    string::substring(ctx, this, start, end)
}

/// String.prototype.indexOf() wrapper
pub fn string_index_of_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let search = args.get(0).copied().unwrap_or(JSValue::undefined());
    let from_index = args.get(1).map(|v| to_int32(ctx, *v));

    let index = string::index_of(ctx, this, search, from_index)?;
    ctx.new_number(index as f64)
        .map_err(|_| JSValue::exception())
}

/// String.prototype.lastIndexOf() wrapper
pub fn string_last_index_of_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let search = args.get(0).copied().unwrap_or(JSValue::undefined());
    let from_index = args.get(1).map(|v| to_int32(ctx, *v));

    let index = string::last_index_of(ctx, this, search, from_index)?;
    ctx.new_number(index as f64)
        .map_err(|_| JSValue::exception())
}

/// String.prototype.toLowerCase() wrapper
pub fn string_to_lower_case_native(ctx: &mut Context, this: JSValue, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    string::to_lower_case(ctx, this)
}

/// String.prototype.toUpperCase() wrapper
pub fn string_to_upper_case_native(ctx: &mut Context, this: JSValue, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    string::to_upper_case(ctx, this)
}

/// String.prototype.split() wrapper
pub fn string_split_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let separator = args.get(0).copied();
    let limit = args.get(1).map(|v| to_int32(ctx, *v));

    string::split(ctx, this, separator, limit)
}

/// String.prototype.trim() wrapper
pub fn string_trim_native(ctx: &mut Context, this: JSValue, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    string::trim(ctx, this)
}

// ========== Object Static Methods ==========

/// Object.keys() wrapper
pub fn object_keys_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let obj = args.get(0).copied().unwrap_or(JSValue::undefined());
    object::object_keys(ctx, obj)
}

/// Object.values() wrapper
pub fn object_values_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let obj = args.get(0).copied().unwrap_or(JSValue::undefined());
    object::object_values(ctx, obj)
}

/// Object.entries() wrapper
pub fn object_entries_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let obj = args.get(0).copied().unwrap_or(JSValue::undefined());
    object::object_entries(ctx, obj)
}

/// Object.assign() wrapper
pub fn object_assign_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    if args.is_empty() {
        return Err(JSValue::exception());
    }

    let target = args[0];
    let sources = &args[1..];

    object::object_assign(ctx, target, sources)
}

// ========== Global Functions ==========

/// parseInt() wrapper
pub fn parse_int_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::globals::parse_int;
    use crate::runtime::conversion::to_int32;

    let string = args.get(0).copied().unwrap_or(JSValue::undefined());
    let radix = args.get(1).map(|v| to_int32(ctx, *v));

    Ok(parse_int(ctx, string, radix))
}

/// parseFloat() wrapper
pub fn parse_float_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::globals::parse_float;

    let string = args.get(0).copied().unwrap_or(JSValue::undefined());
    Ok(parse_float(ctx, string))
}

/// isNaN() wrapper
pub fn is_nan_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::globals::is_nan;

    let value = args.get(0).copied().unwrap_or(JSValue::undefined());
    Ok(JSValue::bool(is_nan(ctx, value)))
}

/// isFinite() wrapper
pub fn is_finite_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::globals::is_finite;

    let value = args.get(0).copied().unwrap_or(JSValue::undefined());
    Ok(JSValue::bool(is_finite(ctx, value)))
}

/// encodeURI() wrapper
pub fn encode_uri_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::globals::encode_uri;

    let uri = args.get(0).copied().unwrap_or(JSValue::undefined());
    encode_uri(ctx, uri)
}

/// decodeURI() wrapper
pub fn decode_uri_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::globals::decode_uri;

    let uri = args.get(0).copied().unwrap_or(JSValue::undefined());
    decode_uri(ctx, uri)
}

/// encodeURIComponent() wrapper
pub fn encode_uri_component_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::globals::encode_uri_component;

    let component = args.get(0).copied().unwrap_or(JSValue::undefined());
    encode_uri_component(ctx, component)
}

/// decodeURIComponent() wrapper
pub fn decode_uri_component_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::globals::decode_uri_component;

    let component = args.get(0).copied().unwrap_or(JSValue::undefined());
    decode_uri_component(ctx, component)
}

// ========== Function.prototype Methods ==========

/// Function.prototype.call() wrapper
pub fn function_call_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let this_arg = args.get(0).copied().unwrap_or(JSValue::undefined());
    let call_args = if args.len() > 1 { &args[1..] } else { &[] };

    // Call the function directly using ctx.call_function
    ctx.call_function(this, this_arg, call_args)
}

/// Function.prototype.apply() wrapper
pub fn function_apply_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use alloc::vec::Vec;

    let this_arg = args.get(0).copied().unwrap_or(JSValue::undefined());
    let args_array = args.get(1).copied().unwrap_or(JSValue::undefined());

    // Convert args array to a Vec to avoid borrowing issues
    let call_args_vec: Vec<JSValue> = if let Some(arr_idx) = args_array.to_ptr() {
        if let Some(arr) = ctx.get_value_array(arr_idx) {
            unsafe { arr.as_slice().to_vec() }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Call the function
    ctx.call_function(this, this_arg, &call_args_vec)
}

/// Function.prototype.bind() wrapper (simplified - returns original function for now)
pub fn function_bind_native(_ctx: &mut Context, this: JSValue, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    // TODO: Implement proper bound function creation
    // For now, just return the original function
    Ok(this)
}
