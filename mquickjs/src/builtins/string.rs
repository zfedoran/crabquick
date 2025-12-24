//! String built-in constructor and methods
//!
//! Implements String(), String.prototype.length, and String.prototype methods:
//! charAt, charCodeAt, indexOf, lastIndexOf, slice, substring, substr,
//! toLowerCase, toUpperCase, trim, split, replace, includes, startsWith, endsWith

use crate::context::Context;
use crate::value::JSValue;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// String() constructor
///
/// Converts a value to a string
pub fn string_constructor(ctx: &mut Context, value: Option<JSValue>) -> Result<JSValue, JSValue> {
    match value {
        None => ctx.new_string("").map_err(|_| JSValue::exception()),
        Some(val) => {
            // Convert value to string
            if let Some(s) = ctx.get_string(val) {
                let owned = s.to_string();
                ctx.new_string(&owned).map_err(|_| JSValue::exception())
            } else if let Some(n) = ctx.get_number(val) {
                let s = alloc::format!("{}", n);
                ctx.new_string(&s).map_err(|_| JSValue::exception())
            } else if val.is_null() {
                ctx.new_string("null").map_err(|_| JSValue::exception())
            } else if val.is_undefined() {
                ctx.new_string("undefined").map_err(|_| JSValue::exception())
            } else if let Some(b) = val.to_bool() {
                let s = if b { "true" } else { "false" };
                ctx.new_string(s).map_err(|_| JSValue::exception())
            } else {
                ctx.new_string("[object Object]").map_err(|_| JSValue::exception())
            }
        }
    }
}

/// String.prototype.length - Returns the length of a string
pub fn string_length(ctx: &Context, str_val: JSValue) -> Result<i32, JSValue> {
    if let Some(s) = ctx.get_string(str_val) {
        Ok(s.len() as i32)
    } else {
        Err(JSValue::exception())
    }
}

/// String.prototype.charAt() - Returns character at specified index
pub fn char_at(ctx: &mut Context, str_val: JSValue, index: i32) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?;

    if index < 0 || index >= s.len() as i32 {
        return ctx.new_string("").map_err(|_| JSValue::exception());
    }

    let ch = s.chars().nth(index as usize).unwrap_or('\0');
    let mut buf = [0u8; 4];
    let ch_str = ch.encode_utf8(&mut buf);

    ctx.new_string(ch_str).map_err(|_| JSValue::exception())
}

/// String.prototype.charCodeAt() - Returns character code at specified index
pub fn char_code_at(ctx: &Context, str_val: JSValue, index: i32) -> Result<i32, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?;

    if index < 0 || index >= s.len() as i32 {
        return Ok(-1); // Return NaN in real implementation
    }

    let ch = s.chars().nth(index as usize).unwrap_or('\0');
    Ok(ch as i32)
}

/// String.prototype.indexOf() - Returns first index of substring
pub fn index_of(ctx: &Context, str_val: JSValue, search: JSValue, from_index: Option<i32>) -> Result<i32, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?;
    let search_str = ctx.get_string(search).ok_or(JSValue::exception())?;

    let start = from_index.unwrap_or(0).max(0) as usize;

    if start >= s.len() {
        return Ok(-1);
    }

    match s[start..].find(search_str) {
        Some(pos) => Ok((start + pos) as i32),
        None => Ok(-1),
    }
}

/// String.prototype.lastIndexOf() - Returns last index of substring
pub fn last_index_of(ctx: &Context, str_val: JSValue, search: JSValue, from_index: Option<i32>) -> Result<i32, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?;
    let search_str = ctx.get_string(search).ok_or(JSValue::exception())?;

    let end = from_index.map(|i| (i as usize).min(s.len())).unwrap_or(s.len());

    match s[..end].rfind(search_str) {
        Some(pos) => Ok(pos as i32),
        None => Ok(-1),
    }
}

/// String.prototype.slice() - Extracts a section of a string
pub fn slice(ctx: &mut Context, str_val: JSValue, start: i32, end: Option<i32>) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?.to_string();
    let len = s.len() as i32;

    let start_idx = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
    let end_idx = if let Some(e) = end {
        if e < 0 { (len + e).max(0) } else { e.min(len) }
    } else {
        len
    } as usize;

    if start_idx >= end_idx {
        return ctx.new_string("").map_err(|_| JSValue::exception());
    }

    let result = &s[start_idx..end_idx];
    ctx.new_string(result).map_err(|_| JSValue::exception())
}

/// String.prototype.substring() - Returns substring between two indices
pub fn substring(ctx: &mut Context, str_val: JSValue, start: i32, end: Option<i32>) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?.to_string();
    let len = s.len() as i32;

    let start_idx = start.max(0).min(len) as usize;
    let end_idx = end.unwrap_or(len).max(0).min(len) as usize;

    let (start_idx, end_idx) = if start_idx > end_idx {
        (end_idx, start_idx)
    } else {
        (start_idx, end_idx)
    };

    let result = &s[start_idx..end_idx];
    ctx.new_string(result).map_err(|_| JSValue::exception())
}

/// String.prototype.substr() - Returns substring starting at index with length
pub fn substr(ctx: &mut Context, str_val: JSValue, start: i32, length: Option<i32>) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?.to_string();
    let len = s.len() as i32;

    let start_idx = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
    let length = length.unwrap_or(len).max(0) as usize;
    let end_idx = (start_idx + length).min(s.len());

    let result = &s[start_idx..end_idx];
    ctx.new_string(result).map_err(|_| JSValue::exception())
}

/// String.prototype.toLowerCase() - Converts string to lowercase
pub fn to_lower_case(ctx: &mut Context, str_val: JSValue) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?.to_string();
    let lower = s.to_lowercase();
    ctx.new_string(&lower).map_err(|_| JSValue::exception())
}

/// String.prototype.toUpperCase() - Converts string to uppercase
pub fn to_upper_case(ctx: &mut Context, str_val: JSValue) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?.to_string();
    let upper = s.to_uppercase();
    ctx.new_string(&upper).map_err(|_| JSValue::exception())
}

/// String.prototype.trim() - Removes whitespace from both ends
pub fn trim(ctx: &mut Context, str_val: JSValue) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?.to_string();
    let trimmed = s.trim();
    ctx.new_string(trimmed).map_err(|_| JSValue::exception())
}

/// String.prototype.split() - Splits string into array
///
/// Simplified implementation
pub fn split(ctx: &mut Context, str_val: JSValue, separator: Option<JSValue>, limit: Option<i32>) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?.to_string();

    let parts: Vec<String> = if let Some(sep) = separator {
        if let Some(sep_str) = ctx.get_string(sep) {
            s.split(sep_str).map(|p: &str| p.to_string()).collect()
        } else {
            vec![s.clone()]
        }
    } else {
        vec![s.clone()]
    };

    let limit = limit.unwrap_or(i32::MAX) as usize;
    let parts: Vec<String> = parts.into_iter().take(limit).collect();

    // Create array of strings
    let mut arr_elements = Vec::new();
    for part in parts {
        let part_val = ctx.new_string(&part).map_err(|_| JSValue::exception())?;
        arr_elements.push(part_val);
    }

    let arr_idx = ctx.alloc_value_array(arr_elements.len()).map_err(|_| JSValue::exception())?;
    if let Some(arr) = ctx.get_value_array_mut(arr_idx) {
        for elem in arr_elements {
            unsafe { arr.push(elem); }
        }
    }

    Ok(JSValue::from_ptr(arr_idx))
}

/// String.prototype.replace() - Replaces first occurrence
///
/// Simplified implementation
pub fn replace(ctx: &mut Context, str_val: JSValue, search: JSValue, replace_val: JSValue) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?;
    let search_str = ctx.get_string(search).ok_or(JSValue::exception())?;
    let replace_str = ctx.get_string(replace_val).ok_or(JSValue::exception())?;

    let result = s.replacen(search_str, replace_str, 1);
    ctx.new_string(&result).map_err(|_| JSValue::exception())
}

/// String.prototype.includes() - Checks if string contains substring
pub fn includes(ctx: &Context, str_val: JSValue, search: JSValue, position: Option<i32>) -> Result<bool, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?;
    let search_str = ctx.get_string(search).ok_or(JSValue::exception())?;

    let start = position.unwrap_or(0).max(0) as usize;

    if start >= s.len() {
        return Ok(false);
    }

    Ok(s[start..].contains(search_str))
}

/// String.prototype.startsWith() - Checks if string starts with substring
pub fn starts_with(ctx: &Context, str_val: JSValue, search: JSValue, position: Option<i32>) -> Result<bool, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?;
    let search_str = ctx.get_string(search).ok_or(JSValue::exception())?;

    let start = position.unwrap_or(0).max(0) as usize;

    if start >= s.len() {
        return Ok(false);
    }

    Ok(s[start..].starts_with(search_str))
}

/// String.prototype.endsWith() - Checks if string ends with substring
pub fn ends_with(ctx: &Context, str_val: JSValue, search: JSValue, length: Option<i32>) -> Result<bool, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?;
    let search_str = ctx.get_string(search).ok_or(JSValue::exception())?;

    let end = length.map(|l| (l as usize).min(s.len())).unwrap_or(s.len());

    Ok(s[..end].ends_with(search_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_constructor() {
        let mut ctx = Context::new(4096);

        let s = string_constructor(&mut ctx, Some(JSValue::from_int(42))).unwrap();
        assert_eq!(ctx.get_string(s).unwrap(), "42");

        let s = string_constructor(&mut ctx, None).unwrap();
        assert_eq!(ctx.get_string(s).unwrap(), "");
    }

    #[test]
    fn test_string_length() {
        let mut ctx = Context::new(4096);

        let s = ctx.new_string("hello").unwrap();
        assert_eq!(string_length(&ctx, s).unwrap(), 5);
    }

    #[test]
    fn test_char_at() {
        let mut ctx = Context::new(4096);

        let s = ctx.new_string("hello").unwrap();
        let ch = char_at(&mut ctx, s, 1).unwrap();
        assert_eq!(ctx.get_string(ch).unwrap(), "e");
    }

    #[test]
    fn test_index_of() {
        let mut ctx = Context::new(4096);

        let s = ctx.new_string("hello world").unwrap();
        let search = ctx.new_string("world").unwrap();
        assert_eq!(index_of(&ctx, s, search, None).unwrap(), 6);
    }

    #[test]
    fn test_slice() {
        let mut ctx = Context::new(4096);

        let s = ctx.new_string("hello").unwrap();
        let result = slice(&mut ctx, s, 1, Some(4)).unwrap();
        assert_eq!(ctx.get_string(result).unwrap(), "ell");
    }

    #[test]
    fn test_to_lower_case() {
        let mut ctx = Context::new(4096);

        let s = ctx.new_string("HELLO").unwrap();
        let result = to_lower_case(&mut ctx, s).unwrap();
        assert_eq!(ctx.get_string(result).unwrap(), "hello");
    }

    #[test]
    fn test_to_upper_case() {
        let mut ctx = Context::new(4096);

        let s = ctx.new_string("hello").unwrap();
        let result = to_upper_case(&mut ctx, s).unwrap();
        assert_eq!(ctx.get_string(result).unwrap(), "HELLO");
    }

    #[test]
    fn test_trim() {
        let mut ctx = Context::new(4096);

        let s = ctx.new_string("  hello  ").unwrap();
        let result = trim(&mut ctx, s).unwrap();
        assert_eq!(ctx.get_string(result).unwrap(), "hello");
    }

    #[test]
    fn test_includes() {
        let mut ctx = Context::new(4096);

        let s = ctx.new_string("hello world").unwrap();
        let search = ctx.new_string("world").unwrap();
        assert!(includes(&ctx, s, search, None).unwrap());

        let search = ctx.new_string("foo").unwrap();
        assert!(!includes(&ctx, s, search, None).unwrap());
    }

    #[test]
    fn test_starts_with() {
        let mut ctx = Context::new(4096);

        let s = ctx.new_string("hello world").unwrap();
        let search = ctx.new_string("hello").unwrap();
        assert!(starts_with(&ctx, s, search, None).unwrap());
    }

    #[test]
    fn test_ends_with() {
        let mut ctx = Context::new(4096);

        let s = ctx.new_string("hello world").unwrap();
        let search = ctx.new_string("world").unwrap();
        assert!(ends_with(&ctx, s, search, None).unwrap());
    }
}
