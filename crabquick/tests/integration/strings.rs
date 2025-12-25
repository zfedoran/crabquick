//! Integration tests for strings

#![cfg(test)]

use crate::harness::*;

#[test]
#[ignore]
fn test_string_literal() {
    assert_js_eq("\"hello\"", "hello");
}

#[test]
#[ignore]
fn test_string_length() {
    let code = r#"
        var s = "hello";
        s.length
    "#;
    assert_js_eq(code, "5");
}

#[test]
#[ignore]
fn test_string_concat() {
    assert_js_eq("\"hello\" + \" \" + \"world\"", "hello world");
}

#[test]
#[ignore]
fn test_string_char_at() {
    let code = r#"
        var s = "hello";
        s.charAt(0)
    "#;
    assert_js_eq(code, "h");
}

#[test]
#[ignore]
fn test_string_char_code_at() {
    let code = r#"
        var s = "A";
        s.charCodeAt(0)
    "#;
    assert_js_eq(code, "65");
}

#[test]
#[ignore]
fn test_string_slice() {
    let code = r#"
        var s = "Hello, World!";
        s.slice(0, 5)
    "#;
    assert_js_eq(code, "Hello");
}

#[test]
#[ignore]
fn test_string_substring() {
    let code = r#"
        var s = "JavaScript";
        s.substring(0, 4)
    "#;
    assert_js_eq(code, "Java");
}

#[test]
#[ignore]
fn test_string_to_upper_case() {
    let code = r#"
        var s = "hello";
        s.toUpperCase()
    "#;
    assert_js_eq(code, "HELLO");
}

#[test]
#[ignore]
fn test_string_to_lower_case() {
    let code = r#"
        var s = "WORLD";
        s.toLowerCase()
    "#;
    assert_js_eq(code, "world");
}

#[test]
#[ignore]
fn test_string_index_of() {
    let code = r#"
        var s = "Hello, World!";
        s.indexOf("World")
    "#;
    assert_js_eq(code, "7");
}

#[test]
#[ignore]
fn test_string_index_of_not_found() {
    let code = r#"
        var s = "hello";
        s.indexOf("xyz")
    "#;
    assert_js_eq(code, "-1");
}

#[test]
#[ignore]
fn test_string_split() {
    let code = r#"
        var s = "a,b,c";
        var arr = s.split(",");
        arr.length
    "#;
    assert_js_eq(code, "3");
}

#[test]
#[ignore]
fn test_string_trim() {
    let code = r#"
        var s = "  hello  ";
        s.trim()
    "#;
    assert_js_eq(code, "hello");
}

#[test]
#[ignore]
fn test_string_replace() {
    let code = r#"
        var s = "hello world";
        s.replace("world", "rust")
    "#;
    assert_js_eq(code, "hello rust");
}

#[test]
#[ignore]
fn test_empty_string() {
    let code = r#"
        var s = "";
        s.length
    "#;
    assert_js_eq(code, "0");
}

#[test]
#[ignore]
fn test_string_comparison() {
    assert_js_true("\"hello\" === \"hello\"");
    assert_js_false("\"hello\" === \"world\"");
}
