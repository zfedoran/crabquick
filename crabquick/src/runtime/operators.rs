//! Operator implementations

use crate::value::JSValue;
use crate::context::Context;
use crate::runtime::conversion::{to_number, to_string};

/// Addition operator (ES5 11.6.1)
///
/// # Rules
/// - If either operand is a string, convert both to strings and concatenate
/// - Otherwise, convert both to numbers and add
pub fn add(ctx: &mut Context, left: JSValue, right: JSValue) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
    // Check if either operand is a string
    let is_left_string = ctx.get_string(left).is_some();
    let is_right_string = ctx.get_string(right).is_some();

    if is_left_string || is_right_string {
        // String concatenation
        let left_str = to_string(ctx, left);
        let right_str = to_string(ctx, right);

        let mut result = left_str;
        result.push_str(&right_str);

        ctx.new_string(&result)
    } else {
        // Numeric addition
        let left_num = to_number(ctx, left);
        let right_num = to_number(ctx, right);
        let sum = left_num + right_num;

        ctx.new_number(sum)
    }
}

/// Subtraction operator
///
/// Converts both operands to numbers and subtracts
pub fn subtract(ctx: &mut Context, left: JSValue, right: JSValue) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
    let left_num = to_number(ctx, left);
    let right_num = to_number(ctx, right);
    let diff = left_num - right_num;

    ctx.new_number(diff)
}

/// Multiplication operator
///
/// Converts both operands to numbers and multiplies
pub fn multiply(ctx: &mut Context, left: JSValue, right: JSValue) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
    let left_num = to_number(ctx, left);
    let right_num = to_number(ctx, right);
    let product = left_num * right_num;

    ctx.new_number(product)
}

/// Division operator
///
/// Converts both operands to numbers and divides
pub fn divide(ctx: &mut Context, left: JSValue, right: JSValue) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
    let left_num = to_number(ctx, left);
    let right_num = to_number(ctx, right);
    let quotient = left_num / right_num;

    ctx.new_number(quotient)
}
