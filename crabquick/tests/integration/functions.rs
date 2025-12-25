//! Integration tests for functions

#![cfg(test)]

use crate::harness::*;

#[test]
#[ignore]
fn test_simple_function() {
    let code = r#"
        function add(a, b) {
            return a + b;
        }
        add(2, 3)
    "#;
    assert_js_eq(code, "5");
}

#[test]
#[ignore]
fn test_function_no_args() {
    let code = r#"
        function getValue() {
            return 42;
        }
        getValue()
    "#;
    assert_js_eq(code, "42");
}

#[test]
#[ignore]
fn test_function_multiple_args() {
    let code = r#"
        function sum(a, b, c) {
            return a + b + c;
        }
        sum(1, 2, 3)
    "#;
    assert_js_eq(code, "6");
}

#[test]
#[ignore]
fn test_recursive_function() {
    let code = r#"
        function factorial(n) {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5)
    "#;
    assert_js_eq(code, "120");
}

#[test]
#[ignore]
fn test_fibonacci() {
    let code = r#"
        function fibonacci(n) {
            if (n <= 1) {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        fibonacci(7)
    "#;
    assert_js_eq(code, "13");
}

#[test]
#[ignore]
fn test_closure() {
    let code = r#"
        function makeCounter() {
            var count = 0;
            return function() {
                count = count + 1;
                return count;
            };
        }
        var counter = makeCounter();
        counter();
        counter();
        counter()
    "#;
    assert_js_eq(code, "3");
}

#[test]
#[ignore]
fn test_closure_with_parameter() {
    let code = r#"
        function makeAdder(x) {
            return function(y) {
                return x + y;
            };
        }
        var add5 = makeAdder(5);
        add5(3)
    "#;
    assert_js_eq(code, "8");
}

#[test]
#[ignore]
fn test_function_scope() {
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
fn test_function_expression() {
    let code = r#"
        var add = function(a, b) {
            return a + b;
        };
        add(4, 5)
    "#;
    assert_js_eq(code, "9");
}

#[test]
#[ignore]
fn test_nested_functions() {
    let code = r#"
        function outer() {
            function inner() {
                return 42;
            }
            return inner();
        }
        outer()
    "#;
    assert_js_eq(code, "42");
}

#[test]
#[ignore]
fn test_return_without_value() {
    let code = r#"
        function test() {
            return;
        }
        test()
    "#;
    assert_js_eq(code, "undefined");
}

#[test]
#[ignore]
fn test_function_without_return() {
    let code = r#"
        function test() {
            var x = 5;
        }
        test()
    "#;
    assert_js_eq(code, "undefined");
}
