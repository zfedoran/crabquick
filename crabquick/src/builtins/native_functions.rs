//! Native function wrappers for built-in JavaScript functions
//!
//! This module provides wrapper functions that convert JavaScript values
//! to native Rust types, call the appropriate built-in function, and
//! convert the result back to JavaScript values.

use crate::context::Context;
use crate::value::JSValue;
use crate::builtins::{math, console, array, string, object, number};
use alloc::string::ToString;

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

/// Array.prototype.join() wrapper
pub fn array_join_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    // Get separator from first argument, default to ","
    let sep = if let Some(sep_val) = args.get(0) {
        if let Some(s) = ctx.get_string(*sep_val) {
            Some(s.to_string())
        } else {
            None
        }
    } else {
        None
    };

    array::array_join(ctx, this, sep.as_deref())
}

/// Array.prototype.reverse() wrapper
pub fn array_reverse_native(ctx: &mut Context, this: JSValue, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    array::array_reverse(ctx, this)
}

/// Array.isArray() - static method on Array constructor
pub fn array_is_array_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let value = args.get(0).copied().unwrap_or(JSValue::undefined());
    let result = array::is_array(ctx, value);
    Ok(JSValue::bool(result))
}

/// Array.prototype.forEach() wrapper
pub fn array_for_each_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let callback = args.get(0).copied().unwrap_or(JSValue::undefined());
    array::array_for_each(ctx, this, callback)
}

/// Array.prototype.map() wrapper
pub fn array_map_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let callback = args.get(0).copied().unwrap_or(JSValue::undefined());
    array::array_map(ctx, this, callback)
}

/// Array.prototype.filter() wrapper
pub fn array_filter_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let callback = args.get(0).copied().unwrap_or(JSValue::undefined());
    array::array_filter(ctx, this, callback)
}

/// Array.prototype.reduce() wrapper
pub fn array_reduce_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let callback = args.get(0).copied().unwrap_or(JSValue::undefined());
    let initial = args.get(1).copied();
    array::array_reduce(ctx, this, callback, initial)
}

/// Array.prototype.find() wrapper
pub fn array_find_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let callback = args.get(0).copied().unwrap_or(JSValue::undefined());
    array::array_find(ctx, this, callback)
}

/// Array.prototype.findIndex() wrapper
pub fn array_find_index_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let callback = args.get(0).copied().unwrap_or(JSValue::undefined());
    array::array_find_index(ctx, this, callback)
}

/// Array.prototype.some() wrapper
pub fn array_some_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let callback = args.get(0).copied().unwrap_or(JSValue::undefined());
    array::array_some(ctx, this, callback)
}

/// Array.prototype.every() wrapper
pub fn array_every_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let callback = args.get(0).copied().unwrap_or(JSValue::undefined());
    array::array_every(ctx, this, callback)
}

/// Array.prototype.lastIndexOf() wrapper
pub fn array_last_index_of_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let search = args.get(0).copied().unwrap_or(JSValue::undefined());
    let from_index = args.get(1).map(|v| to_int32(ctx, *v));

    let result = array::array_last_index_of(ctx, this, search, from_index)?;
    Ok(JSValue::from_int(result))
}

/// Array.prototype.reduceRight() wrapper
pub fn array_reduce_right_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let callback = args.get(0).copied().unwrap_or(JSValue::undefined());
    let initial = args.get(1).copied();
    array::array_reduce_right(ctx, this, callback, initial)
}

/// Array.prototype.sort() wrapper
pub fn array_sort_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let compare_fn = args.get(0).copied().filter(|v| !v.is_undefined());
    array::array_sort(ctx, this, compare_fn)
}

/// Array.prototype.toString() wrapper
pub fn array_to_string_native(ctx: &mut Context, this: JSValue, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    array::array_to_string(ctx, this)
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

/// String.prototype.trimStart() wrapper
pub fn string_trim_start_native(ctx: &mut Context, this: JSValue, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    string::trim_start(ctx, this)
}

/// String.prototype.trimEnd() wrapper
pub fn string_trim_end_native(ctx: &mut Context, this: JSValue, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    string::trim_end(ctx, this)
}

/// String.prototype.replace() wrapper
pub fn string_replace_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let search = args.get(0).copied().unwrap_or(JSValue::undefined());
    let replace_val = args.get(1).copied().unwrap_or(JSValue::undefined());
    string::replace(ctx, this, search, replace_val)
}

/// String.prototype.replaceAll() wrapper
pub fn string_replace_all_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let search = args.get(0).copied().unwrap_or(JSValue::undefined());
    let replace_val = args.get(1).copied().unwrap_or(JSValue::undefined());
    string::replace_all(ctx, this, search, replace_val)
}

/// String.prototype.includes() wrapper
pub fn string_includes_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let search = args.get(0).copied().unwrap_or(JSValue::undefined());
    let position = args.get(1).map(|v| to_int32(ctx, *v));

    let result = string::includes(ctx, this, search, position)?;
    Ok(JSValue::bool(result))
}

/// String.prototype.startsWith() wrapper
pub fn string_starts_with_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let search = args.get(0).copied().unwrap_or(JSValue::undefined());
    let position = args.get(1).map(|v| to_int32(ctx, *v));

    let result = string::starts_with(ctx, this, search, position)?;
    Ok(JSValue::bool(result))
}

/// String.prototype.endsWith() wrapper
pub fn string_ends_with_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let search = args.get(0).copied().unwrap_or(JSValue::undefined());
    let length = args.get(1).map(|v| to_int32(ctx, *v));

    let result = string::ends_with(ctx, this, search, length)?;
    Ok(JSValue::bool(result))
}

/// String.prototype.concat() wrapper
pub fn string_concat_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    string::concat(ctx, this, args)
}

/// String.prototype.codePointAt() wrapper
pub fn string_code_point_at_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::conversion::to_int32;

    let index = args.get(0).map(|v| to_int32(ctx, *v)).unwrap_or(0);
    string::code_point_at(ctx, this, index)
}

/// String.fromCharCode() wrapper
pub fn string_from_char_code_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    string::from_char_code(ctx, args)
}

/// String.fromCodePoint() wrapper
pub fn string_from_code_point_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    string::from_code_point(ctx, args)
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

/// Object.create() wrapper
pub fn object_create_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let proto = args.get(0).copied().unwrap_or(JSValue::null());
    object::object_create(ctx, proto)
}

/// Object.getPrototypeOf() wrapper
pub fn object_get_prototype_of_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let obj = args.get(0).copied().unwrap_or(JSValue::undefined());
    object::get_prototype_of(ctx, obj)
}

/// Object.setPrototypeOf() wrapper
pub fn object_set_prototype_of_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let obj = args.get(0).copied().unwrap_or(JSValue::undefined());
    let proto = args.get(1).copied().unwrap_or(JSValue::null());
    object::set_prototype_of(ctx, obj, proto)
}

/// Object.defineProperty() wrapper
pub fn object_define_property_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let obj = args.get(0).copied().unwrap_or(JSValue::undefined());
    let prop = args.get(1).copied().unwrap_or(JSValue::undefined());
    let descriptor = args.get(2).copied().unwrap_or(JSValue::undefined());
    object::define_property(ctx, obj, prop, descriptor)
}

/// Object.prototype.hasOwnProperty() wrapper
pub fn object_has_own_property_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    let prop = args.get(0).copied().unwrap_or(JSValue::undefined());

    let prop_atom = if let Some(s) = ctx.get_string(prop) {
        string_to_atom(s)
    } else if let Some(n) = prop.to_int() {
        string_to_atom(&alloc::format!("{}", n))
    } else {
        return Ok(JSValue::bool(false));
    };

    Ok(JSValue::bool(object::has_own_property(ctx, this, prop_atom)))
}

/// Object.prototype.toString() wrapper
pub fn object_to_string_native(ctx: &mut Context, this: JSValue, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    object::to_string(ctx, this)
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
    use crate::runtime::init::string_to_atom;

    let this_arg = args.get(0).copied().unwrap_or(JSValue::undefined());
    let args_array = args.get(1).copied().unwrap_or(JSValue::undefined());

    // Extract elements from the array object
    let call_args_vec: Vec<JSValue> = if args_array.is_object() || args_array.to_ptr().is_some() {
        // Get the length
        let length_atom = string_to_atom("length");
        let length = ctx.get_property(args_array, length_atom)
            .and_then(|v| {
                if let Some(i) = v.to_int() {
                    Some(i as usize)
                } else if let Some(n) = ctx.get_number(v) {
                    Some(n as usize)
                } else {
                    None
                }
            })
            .unwrap_or(0);

        // Extract each element by index
        let mut result = Vec::with_capacity(length);
        for i in 0..length {
            let idx_atom = string_to_atom(&alloc::format!("{}", i));
            let val = ctx.get_property(args_array, idx_atom)
                .unwrap_or(JSValue::undefined());
            result.push(val);
        }
        result
    } else {
        Vec::new()
    };

    // Call the function
    ctx.call_function(this, this_arg, &call_args_vec)
}

/// Function.prototype.bind() wrapper - creates a bound function object
pub fn function_bind_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    // Create a new object to represent the bound function
    let bound_obj = ctx.new_object()
        .map_err(|_| JSValue::exception())?;

    // Store the target function
    let target_atom = string_to_atom("__boundTarget__");
    ctx.add_property(bound_obj, target_atom, this, crate::object::PropertyFlags::empty())
        .map_err(|_| JSValue::exception())?;

    // Store the bound this value
    let bound_this = args.get(0).copied().unwrap_or(JSValue::undefined());
    let this_atom = string_to_atom("__boundThis__");
    ctx.add_property(bound_obj, this_atom, bound_this, crate::object::PropertyFlags::empty())
        .map_err(|_| JSValue::exception())?;

    // Store bound arguments (if any)
    if args.len() > 1 {
        let bound_args = ctx.new_object()
            .map_err(|_| JSValue::exception())?;
        for (i, arg) in args[1..].iter().enumerate() {
            let idx_atom = string_to_atom(&alloc::format!("{}", i));
            ctx.add_property(bound_args, idx_atom, *arg, crate::object::PropertyFlags::empty())
                .map_err(|_| JSValue::exception())?;
        }
        let length_atom = string_to_atom("length");
        let length_val = JSValue::from_int((args.len() - 1) as i32);
        ctx.add_property(bound_args, length_atom, length_val, crate::object::PropertyFlags::empty())
            .map_err(|_| JSValue::exception())?;

        let args_atom = string_to_atom("__boundArgs__");
        ctx.add_property(bound_obj, args_atom, bound_args, crate::object::PropertyFlags::empty())
            .map_err(|_| JSValue::exception())?;
    }

    // Mark this as a bound function (for call_function to recognize)
    let is_bound_atom = string_to_atom("__isBoundFunction__");
    ctx.add_property(bound_obj, is_bound_atom, JSValue::bool(true), crate::object::PropertyFlags::empty())
        .map_err(|_| JSValue::exception())?;

    Ok(bound_obj)
}

// ========== Number Methods ==========

/// Number.isNaN() wrapper
pub fn number_is_nan_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::builtins::number;

    let value = args.get(0).copied().unwrap_or(JSValue::undefined());
    Ok(JSValue::bool(number::is_nan(ctx, value)))
}

/// Number.isFinite() wrapper
pub fn number_is_finite_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::builtins::number;

    let value = args.get(0).copied().unwrap_or(JSValue::undefined());
    Ok(JSValue::bool(number::is_finite(ctx, value)))
}

/// Number.isInteger() wrapper
pub fn number_is_integer_native(ctx: &mut Context, _this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::builtins::number;

    let value = args.get(0).copied().unwrap_or(JSValue::undefined());
    Ok(JSValue::bool(number::is_integer(ctx, value)))
}

/// Number.prototype.toFixed() wrapper
pub fn number_to_fixed_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::builtins::number;
    use crate::runtime::conversion::to_int32;

    let digits = args.get(0).map(|v| to_int32(ctx, *v));
    number::to_fixed(ctx, this, digits)
}

/// Number.prototype.toString() wrapper
pub fn number_to_string_native(ctx: &mut Context, this: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    use crate::builtins::number;
    use crate::runtime::conversion::to_int32;

    let radix = args.get(0).map(|v| to_int32(ctx, *v));
    number::to_string(ctx, this, radix)
}
