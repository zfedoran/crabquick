//! Integration tests for basic expressions
//!
//! Tests arithmetic, comparisons, and logical operators

#![cfg(test)]

use crate::harness::*;

#[test]
#[ignore] // Enable once compiler is fully working
fn test_integer_literals() {
    assert_js_eq("0", "0");
    assert_js_eq("42", "42");
    assert_js_eq("123", "123");
    assert_js_eq("-5", "-5");
}

#[test]
#[ignore]
fn test_float_literals() {
    assert_js_eq("3.14", "3.14");
    assert_js_eq("0.5", "0.5");
    assert_js_eq("-2.5", "-2.5");
}

#[test]
#[ignore]
fn test_boolean_literals() {
    assert_js_eq("true", "true");
    assert_js_eq("false", "false");
}

#[test]
#[ignore]
fn test_null_undefined() {
    assert_js_eq("null", "null");
    assert_js_eq("undefined", "undefined");
}

#[test]
#[ignore]
fn test_string_literals() {
    assert_js_eq("\"hello\"", "hello");
    assert_js_eq("\"world\"", "world");
    assert_js_eq("\"\"", "");
}

#[test]
#[ignore]
fn test_addition() {
    assert_js_eq("1 + 2", "3");
    assert_js_eq("5 + 10", "15");
    assert_js_eq("0 + 0", "0");
    assert_js_eq("-5 + 3", "-2");
}

#[test]
#[ignore]
fn test_subtraction() {
    assert_js_eq("5 - 3", "2");
    assert_js_eq("10 - 15", "-5");
    assert_js_eq("0 - 5", "-5");
}

#[test]
#[ignore]
fn test_multiplication() {
    assert_js_eq("2 * 3", "6");
    assert_js_eq("5 * 10", "50");
    assert_js_eq("0 * 100", "0");
    assert_js_eq("-2 * 3", "-6");
}

#[test]
#[ignore]
fn test_division() {
    assert_js_eq("10 / 2", "5");
    assert_js_eq("15 / 3", "5");
    assert_js_num("10 / 3", 3.3333);
}

#[test]
#[ignore]
fn test_modulo() {
    assert_js_eq("10 % 3", "1");
    assert_js_eq("15 % 4", "3");
    assert_js_eq("8 % 2", "0");
}

#[test]
#[ignore]
fn test_operator_precedence() {
    assert_js_eq("1 + 2 * 3", "7");
    assert_js_eq("(1 + 2) * 3", "9");
    assert_js_eq("10 - 3 * 2", "4");
    assert_js_eq("(10 - 3) * 2", "14");
    assert_js_eq("2 + 3 * 4 - 5", "9");
}

#[test]
#[ignore]
fn test_unary_operators() {
    assert_js_eq("-5", "-5");
    assert_js_eq("-(-5)", "5");
    assert_js_eq("+42", "42");
}

#[test]
#[ignore]
fn test_logical_not() {
    assert_js_eq("!true", "false");
    assert_js_eq("!false", "true");
    assert_js_eq("!!true", "true");
}

#[test]
#[ignore]
fn test_equality() {
    assert_js_true("1 === 1");
    assert_js_true("5 === 5");
    assert_js_false("1 === 2");
    assert_js_true("true === true");
    assert_js_false("true === false");
}

#[test]
#[ignore]
fn test_inequality() {
    assert_js_true("1 !== 2");
    assert_js_false("5 !== 5");
}

#[test]
#[ignore]
fn test_comparison_operators() {
    assert_js_true("1 < 2");
    assert_js_false("2 < 1");
    assert_js_false("2 < 2");

    assert_js_true("2 > 1");
    assert_js_false("1 > 2");
    assert_js_false("2 > 2");

    assert_js_true("1 <= 2");
    assert_js_true("2 <= 2");
    assert_js_false("3 <= 2");

    assert_js_true("2 >= 1");
    assert_js_true("2 >= 2");
    assert_js_false("1 >= 2");
}

#[test]
#[ignore]
fn test_logical_and() {
    assert_js_true("true && true");
    assert_js_false("true && false");
    assert_js_false("false && true");
    assert_js_false("false && false");
}

#[test]
#[ignore]
fn test_logical_or() {
    assert_js_true("true || true");
    assert_js_true("true || false");
    assert_js_true("false || true");
    assert_js_false("false || false");
}

#[test]
#[ignore]
fn test_complex_expressions() {
    assert_js_eq("(2 + 3) * (4 - 1)", "15");
    assert_js_eq("10 / 2 + 3 * 4", "17");
    assert_js_true("(1 + 2) === 3");
    assert_js_true("5 > 3 && 10 < 20");
    assert_js_true("false || true && true");
}
