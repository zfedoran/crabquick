//! JSON built-in functions
//!
//! Implements JSON.parse and JSON.stringify

use alloc::string::String;
use alloc::vec::Vec;
use crate::context::Context;
use crate::value::JSValue;
use crate::object::PropertyFlags;

/// JSON.parse() - Parses a JSON string and returns a JavaScript value
pub fn parse(ctx: &mut Context, json_str: &str) -> Result<JSValue, JSValue> {
    let mut parser = JsonParser::new(json_str);
    parser.parse(ctx)
}

/// JSON.stringify() - Converts a JavaScript value to a JSON string
pub fn stringify(ctx: &Context, value: JSValue) -> Result<String, JSValue> {
    let mut result = String::new();
    stringify_value(ctx, value, &mut result)?;
    Ok(result)
}

// ========== JSON Parser ==========

struct JsonParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        JsonParser { input, pos: 0 }
    }

    fn parse(&mut self, ctx: &mut Context) -> Result<JSValue, JSValue> {
        self.skip_whitespace();
        let result = self.parse_value(ctx)?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(ctx.new_string("Unexpected characters after JSON")
                .unwrap_or(JSValue::exception()));
        }
        Ok(result)
    }

    fn parse_value(&mut self, ctx: &mut Context) -> Result<JSValue, JSValue> {
        self.skip_whitespace();

        match self.peek() {
            Some('"') => self.parse_string(ctx),
            Some('{') => self.parse_object(ctx),
            Some('[') => self.parse_array(ctx),
            Some('t') => self.parse_true(ctx),
            Some('f') => self.parse_false(ctx),
            Some('n') => self.parse_null(ctx),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(ctx),
            _ => Err(ctx.new_string("Unexpected character in JSON")
                .unwrap_or(JSValue::exception())),
        }
    }

    fn parse_string(&mut self, ctx: &mut Context) -> Result<JSValue, JSValue> {
        self.expect('"')?;
        let mut s = String::new();

        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance();
                return ctx.new_string(&s).map_err(|_| JSValue::exception());
            } else if c == '\\' {
                self.advance();
                match self.peek() {
                    Some('"') => { s.push('"'); self.advance(); }
                    Some('\\') => { s.push('\\'); self.advance(); }
                    Some('/') => { s.push('/'); self.advance(); }
                    Some('b') => { s.push('\x08'); self.advance(); }
                    Some('f') => { s.push('\x0c'); self.advance(); }
                    Some('n') => { s.push('\n'); self.advance(); }
                    Some('r') => { s.push('\r'); self.advance(); }
                    Some('t') => { s.push('\t'); self.advance(); }
                    Some('u') => {
                        self.advance();
                        let hex = self.take_n(4);
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = char::from_u32(code) {
                                s.push(ch);
                            }
                        }
                    }
                    _ => return Err(ctx.new_string("Invalid escape sequence")
                        .unwrap_or(JSValue::exception())),
                }
            } else {
                s.push(c);
                self.advance();
            }
        }

        Err(ctx.new_string("Unterminated string").unwrap_or(JSValue::exception()))
    }

    fn parse_number(&mut self, ctx: &mut Context) -> Result<JSValue, JSValue> {
        let start = self.pos;

        // Optional minus
        if self.peek() == Some('-') {
            self.advance();
        }

        // Integer part
        if self.peek() == Some('0') {
            self.advance();
        } else {
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Fractional part
        if self.peek() == Some('.') {
            self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Exponent part
        if self.peek() == Some('e') || self.peek() == Some('E') {
            self.advance();
            if self.peek() == Some('+') || self.peek() == Some('-') {
                self.advance();
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(n) => {
                if n.fract() == 0.0 && n >= i32::MIN as f64 && n <= i32::MAX as f64 {
                    Ok(JSValue::from_int(n as i32))
                } else {
                    ctx.new_number(n).map_err(|_| JSValue::exception())
                }
            }
            Err(_) => Err(ctx.new_string("Invalid number").unwrap_or(JSValue::exception())),
        }
    }

    fn parse_object(&mut self, ctx: &mut Context) -> Result<JSValue, JSValue> {
        use crate::runtime::init::string_to_atom;

        self.expect('{')?;
        self.skip_whitespace();

        let obj = ctx.new_object().map_err(|_| JSValue::exception())?;

        if self.peek() == Some('}') {
            self.advance();
            return Ok(obj);
        }

        loop {
            self.skip_whitespace();

            // Parse key (must be string)
            if self.peek() != Some('"') {
                return Err(ctx.new_string("Expected string key").unwrap_or(JSValue::exception()));
            }
            let key_val = self.parse_string(ctx)?;
            let key_str = ctx.get_string(key_val)
                .ok_or(JSValue::exception())?;
            let key_atom = string_to_atom(key_str);

            self.skip_whitespace();
            self.expect(':')?;
            self.skip_whitespace();

            // Parse value
            let value = self.parse_value(ctx)?;

            // Add to object
            ctx.add_property(obj, key_atom, value, PropertyFlags::default())
                .map_err(|_| JSValue::exception())?;

            self.skip_whitespace();
            match self.peek() {
                Some(',') => { self.advance(); }
                Some('}') => { self.advance(); return Ok(obj); }
                _ => return Err(ctx.new_string("Expected ',' or '}'").unwrap_or(JSValue::exception())),
            }
        }
    }

    fn parse_array(&mut self, ctx: &mut Context) -> Result<JSValue, JSValue> {
        use crate::runtime::init::string_to_atom;

        self.expect('[')?;
        self.skip_whitespace();

        let arr = ctx.new_object().map_err(|_| JSValue::exception())?;
        let mut index = 0;

        if self.peek() == Some(']') {
            self.advance();
            let length_atom = string_to_atom("length");
            ctx.add_property(arr, length_atom, JSValue::from_int(0), PropertyFlags::default())
                .map_err(|_| JSValue::exception())?;
            return Ok(arr);
        }

        loop {
            self.skip_whitespace();
            let value = self.parse_value(ctx)?;

            let idx_atom = string_to_atom(&alloc::format!("{}", index));
            ctx.add_property(arr, idx_atom, value, PropertyFlags::default())
                .map_err(|_| JSValue::exception())?;
            index += 1;

            self.skip_whitespace();
            match self.peek() {
                Some(',') => { self.advance(); }
                Some(']') => {
                    self.advance();
                    let length_atom = string_to_atom("length");
                    ctx.add_property(arr, length_atom, JSValue::from_int(index), PropertyFlags::default())
                        .map_err(|_| JSValue::exception())?;
                    return Ok(arr);
                }
                _ => return Err(ctx.new_string("Expected ',' or ']'").unwrap_or(JSValue::exception())),
            }
        }
    }

    fn parse_true(&mut self, ctx: &mut Context) -> Result<JSValue, JSValue> {
        if self.take_n(4) == "true" {
            Ok(JSValue::bool(true))
        } else {
            Err(ctx.new_string("Expected 'true'").unwrap_or(JSValue::exception()))
        }
    }

    fn parse_false(&mut self, ctx: &mut Context) -> Result<JSValue, JSValue> {
        if self.take_n(5) == "false" {
            Ok(JSValue::bool(false))
        } else {
            Err(ctx.new_string("Expected 'false'").unwrap_or(JSValue::exception()))
        }
    }

    fn parse_null(&mut self, ctx: &mut Context) -> Result<JSValue, JSValue> {
        if self.take_n(4) == "null" {
            Ok(JSValue::null())
        } else {
            Err(ctx.new_string("Expected 'null'").unwrap_or(JSValue::exception()))
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), JSValue> {
        if self.peek() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(JSValue::exception())
        }
    }

    fn take_n(&mut self, n: usize) -> String {
        let mut s = String::new();
        for _ in 0..n {
            if let Some(c) = self.peek() {
                s.push(c);
                self.advance();
            }
        }
        s
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
}

// ========== JSON Stringify ==========

fn stringify_value(ctx: &Context, value: JSValue, result: &mut String) -> Result<(), JSValue> {
    // null
    if value.is_null() {
        result.push_str("null");
        return Ok(());
    }

    // undefined -> null in JSON
    if value.is_undefined() {
        result.push_str("null");
        return Ok(());
    }

    // boolean
    if let Some(b) = value.to_bool() {
        result.push_str(if b { "true" } else { "false" });
        return Ok(());
    }

    // integer
    if let Some(i) = value.to_int() {
        result.push_str(&alloc::format!("{}", i));
        return Ok(());
    }

    // float
    if let Some(f) = ctx.get_number(value) {
        if f.is_nan() || f.is_infinite() {
            result.push_str("null");
        } else {
            result.push_str(&alloc::format!("{}", f));
        }
        return Ok(());
    }

    // string
    if let Some(s) = ctx.get_string(value) {
        stringify_string(s, result);
        return Ok(());
    }

    // object or array
    if value.is_object() {
        // Check if it's array-like (has numeric "length" property)
        let length_atom = crate::runtime::init::string_to_atom("length");
        if let Some(len_val) = ctx.get_property(value, length_atom) {
            if let Some(len) = len_val.to_int() {
                // Array-like
                result.push('[');
                for i in 0..len {
                    if i > 0 {
                        result.push(',');
                    }
                    let idx_atom = crate::runtime::init::string_to_atom(&alloc::format!("{}", i));
                    let elem = ctx.get_property(value, idx_atom).unwrap_or(JSValue::undefined());
                    stringify_value(ctx, elem, result)?;
                }
                result.push(']');
                return Ok(());
            }
        }

        // Regular object - for now just output empty object
        // (full object property enumeration would require more API support)
        result.push_str("{}");
        return Ok(());
    }

    // Function or other -> null
    result.push_str("null");
    Ok(())
}

fn stringify_string(s: &str, result: &mut String) {
    result.push('"');
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c < '\x20' => {
                result.push_str(&alloc::format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let mut ctx = Context::new(4096);

        // Numbers
        let result = parse(&mut ctx, "42").unwrap();
        assert_eq!(result.to_int(), Some(42));

        let result = parse(&mut ctx, "-3.14").unwrap();
        assert!(ctx.get_number(result).unwrap() < -3.13);

        // Booleans
        let result = parse(&mut ctx, "true").unwrap();
        assert_eq!(result.to_bool(), Some(true));

        let result = parse(&mut ctx, "false").unwrap();
        assert_eq!(result.to_bool(), Some(false));

        // Null
        let result = parse(&mut ctx, "null").unwrap();
        assert!(result.is_null());

        // String
        let result = parse(&mut ctx, r#""hello""#).unwrap();
        assert_eq!(ctx.get_string(result), Some("hello"));
    }

    #[test]
    fn test_stringify_simple() {
        let ctx = Context::new(4096);

        assert_eq!(stringify(&ctx, JSValue::null()).unwrap(), "null");
        assert_eq!(stringify(&ctx, JSValue::bool(true)).unwrap(), "true");
        assert_eq!(stringify(&ctx, JSValue::bool(false)).unwrap(), "false");
        assert_eq!(stringify(&ctx, JSValue::from_int(42)).unwrap(), "42");
    }
}
