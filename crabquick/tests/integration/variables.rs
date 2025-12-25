//! Integration tests for variables and scoping

#![cfg(test)]

use crate::harness::*;

#[test]
#[ignore]
fn test_var_declaration() {
    assert_js_eq("var x = 10; x", "10");
    assert_js_eq("var y = 5; y", "5");
}

#[test]
#[ignore]
fn test_var_assignment() {
    assert_js_eq("var x = 10; x = 20; x", "20");
    assert_js_eq("var y = 5; y = y + 1; y", "6");
}

#[test]
#[ignore]
fn test_multiple_vars() {
    let code = r#"
        var a = 1;
        var b = 2;
        var c = 3;
        a + b + c
    "#;
    assert_js_eq(code, "6");
}

#[test]
#[ignore]
fn test_var_with_expressions() {
    assert_js_eq("var x = 2 + 3; x", "5");
    assert_js_eq("var y = 10 * 5; y", "50");
}

#[test]
#[ignore]
fn test_var_chain() {
    let code = r#"
        var x = 5;
        var y = x + 10;
        var z = y * 2;
        z
    "#;
    assert_js_eq(code, "30");
}

#[test]
#[ignore]
fn test_reassignment() {
    let code = r#"
        var x = 10;
        x = 20;
        x = 30;
        x
    "#;
    assert_js_eq(code, "30");
}

#[test]
#[ignore]
fn test_undefined_variable() {
    // Accessing undefined variable should return undefined or throw
    // For now we test that it doesn't crash
    let _ = run_js("x");
}

#[test]
#[ignore]
fn test_var_shadowing() {
    let code = r#"
        var x = 10;
        function test() {
            var x = 20;
            return x;
        }
        test()
    "#;
    assert_js_eq(code, "20");
}

#[test]
#[ignore]
fn test_global_scope() {
    let code = r#"
        var global = 100;
        function test() {
            return global;
        }
        test()
    "#;
    assert_js_eq(code, "100");
}
