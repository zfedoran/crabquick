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
    use crate::runtime::init::string_to_atom;
    use crate::object::PropertyFlags;

    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?.to_string();

    let parts: Vec<String> = if let Some(sep) = separator {
        if let Some(sep_str) = ctx.get_string(sep) {
            if sep_str.is_empty() {
                // Split into individual characters
                s.chars().map(|c| c.to_string()).collect()
            } else {
                s.split(sep_str).map(|p: &str| p.to_string()).collect()
            }
        } else {
            vec![s.clone()]
        }
    } else {
        vec![s.clone()]
    };

    let limit = limit.unwrap_or(i32::MAX) as usize;
    let parts: Vec<String> = parts.into_iter().take(limit).collect();

    // Create a proper JS array object with Array.prototype
    let result = ctx.new_object().map_err(|_| JSValue::exception())?;

    // Set Array.prototype
    let array_atom = string_to_atom("Array");
    let proto_atom = string_to_atom("prototype");
    if let Some(array_ctor) = ctx.get_global_property(array_atom) {
        if let Some(array_proto) = ctx.get_property(array_ctor, proto_atom) {
            if let Some(obj) = ctx.get_object_mut(result) {
                obj.set_prototype(array_proto);
            }
        }
    }

    // Add each part as a numbered property
    for (i, part) in parts.iter().enumerate() {
        let part_val = ctx.new_string(part).map_err(|_| JSValue::exception())?;
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);
        ctx.add_property(result, idx_atom, part_val, PropertyFlags::default())
            .map_err(|_| JSValue::exception())?;
    }

    // Set length
    let length_atom = string_to_atom("length");
    let length_val = JSValue::from_int(parts.len() as i32);
    ctx.add_property(result, length_atom, length_val, PropertyFlags::default())
        .map_err(|_| JSValue::exception())?;

    Ok(result)
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

/// String.prototype.trimStart() - Removes whitespace from beginning
pub fn trim_start(ctx: &mut Context, str_val: JSValue) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?.to_string();
    let trimmed = s.trim_start();
    ctx.new_string(trimmed).map_err(|_| JSValue::exception())
}

/// String.prototype.trimEnd() - Removes whitespace from end
pub fn trim_end(ctx: &mut Context, str_val: JSValue) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?.to_string();
    let trimmed = s.trim_end();
    ctx.new_string(trimmed).map_err(|_| JSValue::exception())
}

/// String.prototype.replaceAll() - Replaces all occurrences
pub fn replace_all(ctx: &mut Context, str_val: JSValue, search: JSValue, replace_val: JSValue) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?;
    let search_str = ctx.get_string(search).ok_or(JSValue::exception())?;
    let replace_str = ctx.get_string(replace_val).ok_or(JSValue::exception())?;

    let result = s.replace(search_str, replace_str);
    ctx.new_string(&result).map_err(|_| JSValue::exception())
}

/// String.prototype.concat() - Concatenates strings
pub fn concat(ctx: &mut Context, str_val: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
    let mut result = ctx.get_string(str_val).ok_or(JSValue::exception())?.to_string();

    for arg in args {
        if let Some(s) = ctx.get_string(*arg) {
            result.push_str(s);
        } else if let Some(n) = ctx.get_number(*arg) {
            result.push_str(&alloc::format!("{}", n));
        } else if arg.is_null() {
            result.push_str("null");
        } else if arg.is_undefined() {
            result.push_str("undefined");
        } else if let Some(b) = arg.to_bool() {
            result.push_str(if b { "true" } else { "false" });
        } else {
            result.push_str("[object Object]");
        }
    }

    ctx.new_string(&result).map_err(|_| JSValue::exception())
}

/// String.prototype.codePointAt() - Returns code point at position
pub fn code_point_at(ctx: &Context, str_val: JSValue, index: i32) -> Result<JSValue, JSValue> {
    let s = ctx.get_string(str_val).ok_or(JSValue::exception())?;

    if index < 0 {
        return Ok(JSValue::undefined());
    }

    // Get character at index (UTF-16 code unit semantics)
    let chars: Vec<char> = s.chars().collect();
    if index as usize >= chars.len() {
        return Ok(JSValue::undefined());
    }

    let ch = chars[index as usize];
    Ok(JSValue::from_int(ch as i32))
}

/// String.fromCharCode() - Creates string from char codes
pub fn from_char_code(ctx: &mut Context, codes: &[JSValue]) -> Result<JSValue, JSValue> {
    let mut result = String::new();

    for code in codes {
        let code_val = if let Some(n) = ctx.get_number(*code) {
            n as u32
        } else if let Some(i) = code.to_int() {
            i as u32
        } else {
            0
        };

        // Mask to 16 bits (UTF-16 code unit)
        let code_unit = (code_val & 0xFFFF) as u16;
        if let Some(ch) = char::from_u32(code_unit as u32) {
            result.push(ch);
        }
    }

    ctx.new_string(&result).map_err(|_| JSValue::exception())
}

/// String.fromCodePoint() - Creates string from code points
pub fn from_code_point(ctx: &mut Context, codes: &[JSValue]) -> Result<JSValue, JSValue> {
    let mut result = String::new();

    for code in codes {
        let code_val = if let Some(n) = ctx.get_number(*code) {
            n as u32
        } else if let Some(i) = code.to_int() {
            i as u32
        } else {
            0
        };

        if let Some(ch) = char::from_u32(code_val) {
            result.push(ch);
        } else {
            return Err(JSValue::exception()); // Invalid code point
        }
    }

    ctx.new_string(&result).map_err(|_| JSValue::exception())
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
