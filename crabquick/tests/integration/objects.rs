//! Integration tests for objects

#![cfg(test)]

use crate::harness::*;

#[test]
#[ignore]
fn test_object_literal() {
    let code = r#"
        var obj = { x: 1, y: 2 };
        obj.x
    "#;
    assert_js_eq(code, "1");
}

#[test]
#[ignore]
fn test_object_property_access() {
    let code = r#"
        var obj = { x: 10, y: 20 };
        obj.y
    "#;
    assert_js_eq(code, "20");
}

#[test]
#[ignore]
fn test_object_property_assignment() {
    let code = r#"
        var obj = { x: 1 };
        obj.x = 10;
        obj.x
    "#;
    assert_js_eq(code, "10");
}

#[test]
#[ignore]
fn test_object_add_property() {
    let code = r#"
        var obj = { x: 1 };
        obj.y = 2;
        obj.y
    "#;
    assert_js_eq(code, "2");
}

#[test]
#[ignore]
fn test_object_nested() {
    let code = r#"
        var obj = {
            inner: {
                value: 42
            }
        };
        obj.inner.value
    "#;
    assert_js_eq(code, "42");
}

#[test]
#[ignore]
fn test_object_method() {
    let code = r#"
        var obj = {
            value: 10,
            getValue: function() {
                return this.value;
            }
        };
        obj.getValue()
    "#;
    assert_js_eq(code, "10");
}

#[test]
#[ignore]
fn test_empty_object() {
    let code = r#"
        var obj = {};
        obj.x = 5;
        obj.x
    "#;
    assert_js_eq(code, "5");
}

#[test]
#[ignore]
fn test_object_keys() {
    let code = r#"
        var obj = { a: 1, b: 2, c: 3 };
        Object.keys(obj).length
    "#;
    assert_js_eq(code, "3");
}

#[test]
#[ignore]
fn test_object_prototype() {
    let code = r#"
        var proto = { x: 10 };
        var obj = Object.create(proto);
        obj.x
    "#;
    assert_js_eq(code, "10");
}
