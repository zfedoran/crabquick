//! Number built-in constructor and methods
//!
//! Implements Number(), Number.isNaN(), Number.isFinite(), Number.isInteger(),
//! Number.parseInt(), Number.parseFloat(), Number.prototype.toString(),
//! Number.prototype.toFixed(), and Number constants

use crate::context::Context;
use crate::value::JSValue;
use alloc::string::String;

/// Number() constructor
pub fn number_constructor(ctx: &mut Context, value: Option<JSValue>) -> Result<JSValue, JSValue> {
    match value {
        None => ctx.new_number(0.0).map_err(|_| JSValue::exception()),
        Some(val) => {
            if let Some(n) = ctx.get_number(val) {
                ctx.new_number(n).map_err(|_| JSValue::exception())
            } else if let Some(s) = ctx.get_string(val) {
                // Try to parse string as number
                match s.parse::<f64>() {
                    Ok(n) => ctx.new_number(n).map_err(|_| JSValue::exception()),
                    Err(_) => ctx.new_number(f64::NAN).map_err(|_| JSValue::exception()),
                }
            } else if val.is_null() {
                ctx.new_number(0.0).map_err(|_| JSValue::exception())
            } else if val.is_undefined() {
                ctx.new_number(f64::NAN).map_err(|_| JSValue::exception())
            } else if let Some(b) = val.to_bool() {
                ctx.new_number(if b { 1.0 } else { 0.0 }).map_err(|_| JSValue::exception())
            } else {
                ctx.new_number(f64::NAN).map_err(|_| JSValue::exception())
            }
        }
    }
}

/// Number.isNaN() - Determines whether value is NaN
pub fn is_nan(ctx: &Context, value: JSValue) -> bool {
    if let Some(n) = ctx.get_number(value) {
        n.is_nan()
    } else {
        false
    }
}

/// Number.isFinite() - Determines whether value is a finite number
pub fn is_finite(ctx: &Context, value: JSValue) -> bool {
    if let Some(n) = ctx.get_number(value) {
        n.is_finite()
    } else {
        false
    }
}

/// Number.isInteger() - Determines whether value is an integer
pub fn is_integer(ctx: &Context, value: JSValue) -> bool {
    if let Some(n) = ctx.get_number(value) {
        n.is_finite() && n.fract() == 0.0
    } else {
        false
    }
}

/// Number.parseInt() - Parses a string and returns an integer
pub fn parse_int(s: &str, radix: Option<i32>) -> i32 {
    let radix = radix.unwrap_or(10).clamp(2, 36);
    i32::from_str_radix(s, radix as u32).unwrap_or(0)
}

/// Number.parseFloat() - Parses a string and returns a float
pub fn parse_float(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(f64::NAN)
}

/// Number.prototype.toString() - Returns string representation
pub fn to_string(ctx: &mut Context, num: JSValue, radix: Option<i32>) -> Result<JSValue, JSValue> {
    let n = ctx.get_number(num).ok_or(JSValue::exception())?;

    if n.is_nan() {
        return ctx.new_string("NaN").map_err(|_| JSValue::exception());
    }

    if n.is_infinite() {
        let s = if n > 0.0 { "Infinity" } else { "-Infinity" };
        return ctx.new_string(s).map_err(|_| JSValue::exception());
    }

    let radix = radix.unwrap_or(10);
    if radix == 10 {
        let s = alloc::format!("{}", n);
        ctx.new_string(&s).map_err(|_| JSValue::exception())
    } else {
        // Simplified: only support base 10 for now
        let s = alloc::format!("{}", n as i64);
        ctx.new_string(&s).map_err(|_| JSValue::exception())
    }
}

/// Number.prototype.toFixed() - Formats number with fixed decimal places
pub fn to_fixed(ctx: &mut Context, num: JSValue, digits: Option<i32>) -> Result<JSValue, JSValue> {
    let n = ctx.get_number(num).ok_or(JSValue::exception())?;
    let digits = digits.unwrap_or(0).clamp(0, 20);

    if n.is_nan() {
        return ctx.new_string("NaN").map_err(|_| JSValue::exception());
    }

    if n.is_infinite() {
        let s = if n > 0.0 { "Infinity" } else { "-Infinity" };
        return ctx.new_string(s).map_err(|_| JSValue::exception());
    }

    let s = alloc::format!("{:.prec$}", n, prec = digits as usize);
    ctx.new_string(&s).map_err(|_| JSValue::exception())
}

/// Number constants
pub const MAX_VALUE: f64 = f64::MAX;
pub const MIN_VALUE: f64 = f64::MIN_POSITIVE;
pub const NAN: f64 = f64::NAN;
pub const POSITIVE_INFINITY: f64 = f64::INFINITY;
pub const NEGATIVE_INFINITY: f64 = f64::NEG_INFINITY;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_constructor() {
        let mut ctx = Context::new(4096);

        let n = number_constructor(&mut ctx, Some(JSValue::from_int(42))).unwrap();
        assert_eq!(ctx.get_number(n), Some(42.0));

        let n = number_constructor(&mut ctx, None).unwrap();
        assert_eq!(ctx.get_number(n), Some(0.0));
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

        let finite = ctx.new_number(42.0).unwrap();
        assert!(is_finite(&ctx, finite));

        let inf = ctx.new_number(f64::INFINITY).unwrap();
        assert!(!is_finite(&ctx, inf));
    }

    #[test]
    fn test_is_integer() {
        let mut ctx = Context::new(4096);

        let int_val = ctx.new_number(42.0).unwrap();
        assert!(is_integer(&ctx, int_val));

        let float_val = ctx.new_number(3.14).unwrap();
        assert!(!is_integer(&ctx, float_val));
    }

    #[test]
    fn test_to_fixed() {
        let mut ctx = Context::new(4096);

        let n = ctx.new_number(3.14159).unwrap();
        let result = to_fixed(&mut ctx, n, Some(2)).unwrap();
        assert_eq!(ctx.get_string(result).unwrap(), "3.14");
    }
}
