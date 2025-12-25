//! Comparison operations

use crate::value::JSValue;
use crate::context::Context;
use crate::runtime::conversion::to_number;

/// Strict equality (===) (ES5 11.9.6)
///
/// Compares values without type coercion
pub fn strict_equal(ctx: &Context, left: JSValue, right: JSValue) -> bool {
    // If types are different, return false
    // Check for same special values
    if left.is_undefined() {
        return right.is_undefined();
    }
    if left.is_null() {
        return right.is_null();
    }

    // Check booleans
    if let Some(lb) = left.to_bool() {
        if let Some(rb) = right.to_bool() {
            return lb == rb;
        }
        return false;
    }

    // Check integers
    if let Some(li) = left.to_int() {
        if let Some(ri) = right.to_int() {
            return li == ri;
        }
        // Could be comparing int to boxed float
        if let Some(rf) = ctx.get_number(right) {
            return (li as f64) == rf;
        }
        return false;
    }

    // Check boxed numbers
    if let Some(lf) = ctx.get_number(left) {
        if let Some(ri) = right.to_int() {
            return lf == (ri as f64);
        }
        if let Some(rf) = ctx.get_number(right) {
            // NaN !== NaN in JavaScript
            if lf.is_nan() || rf.is_nan() {
                return false;
            }
            return lf == rf;
        }
        return false;
    }

    // Check strings
    if let Some(ls) = ctx.get_string(left) {
        if let Some(rs) = ctx.get_string(right) {
            return ls == rs;
        }
        return false;
    }

    // For objects/pointers, compare by reference
    if left.is_ptr() && right.is_ptr() {
        return left.as_raw() == right.as_raw();
    }

    false
}

/// Abstract equality (==) (ES5 11.9.3)
///
/// Compares values with type coercion
///
/// # Rules
/// - Same type: use strict equality
/// - null == undefined → true
/// - Number == String → convert string to number
/// - Boolean == anything → convert boolean to number first
/// - Object == primitive → convert object to primitive (not fully implemented)
pub fn abstract_equal(ctx: &Context, left: JSValue, right: JSValue) -> bool {
    // If same type, use strict equality
    if same_type(ctx, left, right) {
        return strict_equal(ctx, left, right);
    }

    // null == undefined
    if (left.is_null() && right.is_undefined()) || (left.is_undefined() && right.is_null()) {
        return true;
    }

    // Number == String: convert string to number
    let left_is_num = left.to_int().is_some() || ctx.get_number(left).is_some();
    let right_is_num = right.to_int().is_some() || ctx.get_number(right).is_some();
    let left_is_str = ctx.get_string(left).is_some();
    let right_is_str = ctx.get_string(right).is_some();

    if left_is_num && right_is_str {
        let left_num = to_number(ctx, left);
        let right_num = to_number(ctx, right);
        return compare_numbers(left_num, right_num);
    }

    if left_is_str && right_is_num {
        let left_num = to_number(ctx, left);
        let right_num = to_number(ctx, right);
        return compare_numbers(left_num, right_num);
    }

    // Boolean == anything: convert boolean to number
    if left.is_bool() {
        let left_num = to_number(ctx, left);
        let left_as_num = if left_num == 0.0 {
            JSValue::from_int(0)
        } else {
            JSValue::from_int(1)
        };
        return abstract_equal(ctx, left_as_num, right);
    }

    if right.is_bool() {
        let right_num = to_number(ctx, right);
        let right_as_num = if right_num == 0.0 {
            JSValue::from_int(0)
        } else {
            JSValue::from_int(1)
        };
        return abstract_equal(ctx, left, right_as_num);
    }

    // Object == primitive: ToPrimitive (not fully implemented)
    // For now, objects are only equal to themselves
    false
}

/// Checks if two values have the same type
fn same_type(ctx: &Context, left: JSValue, right: JSValue) -> bool {
    if left.is_undefined() && right.is_undefined() {
        return true;
    }
    if left.is_null() && right.is_null() {
        return true;
    }
    if left.is_bool() && right.is_bool() {
        return true;
    }

    // Both numbers
    let left_is_num = left.to_int().is_some() || ctx.get_number(left).is_some();
    let right_is_num = right.to_int().is_some() || ctx.get_number(right).is_some();
    if left_is_num && right_is_num {
        return true;
    }

    // Both strings
    if ctx.get_string(left).is_some() && ctx.get_string(right).is_some() {
        return true;
    }

    // Both objects
    if left.is_ptr() && right.is_ptr() {
        return true;
    }

    false
}

/// Compares two numbers for equality (handles NaN)
fn compare_numbers(left: f64, right: f64) -> bool {
    // NaN != NaN
    if left.is_nan() || right.is_nan() {
        return false;
    }
    left == right
}

/// Less than operator
pub fn less_than(ctx: &Context, left: JSValue, right: JSValue) -> bool {
    // Convert both to numbers
    let left_num = to_number(ctx, left);
    let right_num = to_number(ctx, right);

    // NaN comparisons are always false
    if left_num.is_nan() || right_num.is_nan() {
        return false;
    }

    left_num < right_num
}
