//! Integration tests for control flow statements

#![cfg(test)]

use crate::harness::*;

#[test]
#[ignore]
fn test_if_statement() {
    let code = r#"
        var x = 10;
        if (x > 5) {
            x = 20;
        }
        x
    "#;
    assert_js_eq(code, "20");
}

#[test]
#[ignore]
fn test_if_else() {
    let code = r#"
        var x = 3;
        if (x > 5) {
            x = 10;
        } else {
            x = 5;
        }
        x
    "#;
    assert_js_eq(code, "5");
}

#[test]
#[ignore]
fn test_if_else_if() {
    let code = r#"
        var x = 15;
        var result;
        if (x < 10) {
            result = "small";
        } else if (x < 20) {
            result = "medium";
        } else {
            result = "large";
        }
        result
    "#;
    assert_js_eq(code, "medium");
}

#[test]
#[ignore]
fn test_while_loop() {
    let code = r#"
        var sum = 0;
        var i = 1;
        while (i <= 5) {
            sum = sum + i;
            i = i + 1;
        }
        sum
    "#;
    assert_js_eq(code, "15");
}

#[test]
#[ignore]
fn test_while_loop_break() {
    let code = r#"
        var i = 0;
        while (true) {
            i = i + 1;
            if (i >= 10) {
                break;
            }
        }
        i
    "#;
    assert_js_eq(code, "10");
}

#[test]
#[ignore]
fn test_for_loop() {
    let code = r#"
        var sum = 0;
        for (var i = 1; i <= 5; i = i + 1) {
            sum = sum + i;
        }
        sum
    "#;
    assert_js_eq(code, "15");
}

#[test]
#[ignore]
fn test_for_loop_product() {
    let code = r#"
        var product = 1;
        for (var i = 1; i <= 4; i = i + 1) {
            product = product * i;
        }
        product
    "#;
    assert_js_eq(code, "24");
}

#[test]
#[ignore]
fn test_for_loop_break() {
    let code = r#"
        var i;
        for (i = 0; i < 100; i = i + 1) {
            if (i === 5) {
                break;
            }
        }
        i
    "#;
    assert_js_eq(code, "5");
}

#[test]
#[ignore]
fn test_for_loop_continue() {
    let code = r#"
        var sum = 0;
        for (var i = 0; i < 10; i = i + 1) {
            if (i % 2 === 0) {
                continue;
            }
            sum = sum + i;
        }
        sum
    "#;
    assert_js_eq(code, "25"); // 1+3+5+7+9
}

#[test]
#[ignore]
fn test_nested_loops() {
    let code = r#"
        var sum = 0;
        for (var i = 0; i < 3; i = i + 1) {
            for (var j = 0; j < 3; j = j + 1) {
                sum = sum + 1;
            }
        }
        sum
    "#;
    assert_js_eq(code, "9");
}

#[test]
#[ignore]
fn test_ternary_operator() {
    let code = r#"
        var x = 10;
        var result = x > 5 ? "big" : "small";
        result
    "#;
    assert_js_eq(code, "big");
}

#[test]
#[ignore]
fn test_nested_if() {
    let code = r#"
        var x = 10;
        var y = 5;
        var result;
        if (x > 5) {
            if (y > 3) {
                result = "both";
            } else {
                result = "x only";
            }
        } else {
            result = "neither";
        }
        result
    "#;
    assert_js_eq(code, "both");
}

#[test]
#[ignore]
fn test_fizzbuzz() {
    let code = r#"
        var result = "";
        for (var i = 1; i <= 15; i = i + 1) {
            if (i % 15 === 0) {
                result = result + "FizzBuzz ";
            } else if (i % 3 === 0) {
                result = result + "Fizz ";
            } else if (i % 5 === 0) {
                result = result + "Buzz ";
            }
        }
        result
    "#;
    // Should contain Fizz, Buzz, and FizzBuzz
    assert_js_ok(code);
}
