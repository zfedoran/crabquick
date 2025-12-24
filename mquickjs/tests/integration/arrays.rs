//! Integration tests for arrays

#![cfg(test)]

use crate::harness::*;

#[test]
#[ignore]
fn test_array_literal() {
    let code = r#"
        var arr = [1, 2, 3];
        arr.length
    "#;
    assert_js_eq(code, "3");
}

#[test]
#[ignore]
fn test_array_access() {
    let code = r#"
        var arr = [10, 20, 30];
        arr[0]
    "#;
    assert_js_eq(code, "10");
}

#[test]
#[ignore]
fn test_array_access_last() {
    let code = r#"
        var arr = [1, 2, 3, 4, 5];
        arr[4]
    "#;
    assert_js_eq(code, "5");
}

#[test]
#[ignore]
fn test_array_assignment() {
    let code = r#"
        var arr = [1, 2, 3];
        arr[1] = 99;
        arr[1]
    "#;
    assert_js_eq(code, "99");
}

#[test]
#[ignore]
fn test_array_push() {
    let code = r#"
        var arr = [1, 2, 3];
        arr.push(4);
        arr.length
    "#;
    assert_js_eq(code, "4");
}

#[test]
#[ignore]
fn test_array_pop() {
    let code = r#"
        var arr = [1, 2, 3];
        arr.pop()
    "#;
    assert_js_eq(code, "3");
}

#[test]
#[ignore]
fn test_array_pop_length() {
    let code = r#"
        var arr = [1, 2, 3];
        arr.pop();
        arr.length
    "#;
    assert_js_eq(code, "2");
}

#[test]
#[ignore]
fn test_empty_array() {
    let code = r#"
        var arr = [];
        arr.length
    "#;
    assert_js_eq(code, "0");
}

#[test]
#[ignore]
fn test_array_join() {
    let code = r#"
        var arr = [1, 2, 3];
        arr.join(",")
    "#;
    assert_js_eq(code, "1,2,3");
}

#[test]
#[ignore]
fn test_array_slice() {
    let code = r#"
        var arr = [1, 2, 3, 4, 5];
        var sliced = arr.slice(1, 3);
        sliced.length
    "#;
    assert_js_eq(code, "2");
}

#[test]
#[ignore]
fn test_array_concat() {
    let code = r#"
        var arr1 = [1, 2];
        var arr2 = [3, 4];
        var result = arr1.concat(arr2);
        result.length
    "#;
    assert_js_eq(code, "4");
}

#[test]
#[ignore]
fn test_array_is_array() {
    let code = r#"
        var arr = [1, 2, 3];
        Array.isArray(arr)
    "#;
    assert_js_true(code);
}

#[test]
#[ignore]
fn test_array_mixed_types() {
    let code = r#"
        var arr = [1, "hello", true, null];
        arr.length
    "#;
    assert_js_eq(code, "4");
}

#[test]
#[ignore]
fn test_nested_arrays() {
    let code = r#"
        var arr = [[1, 2], [3, 4]];
        arr[0][1]
    "#;
    assert_js_eq(code, "2");
}
