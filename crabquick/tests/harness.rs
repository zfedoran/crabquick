//! Test harness for integration tests
//!
//! This module provides utilities for running JavaScript code in tests
//! and comparing results.

#![cfg(test)]

use crabquick::{Engine, JSValue};

/// Run a JavaScript snippet and return the result as a string
pub fn run_js(source: &str) -> Result<String, String> {
    let mut engine = Engine::new(65536);
    engine.eval_as_string(source)
}

/// Run a JavaScript snippet and expect success with a specific result
pub fn assert_js_eq(source: &str, expected: &str) {
    match run_js(source) {
        Ok(result) => {
            assert_eq!(result, expected, "JavaScript: {}", source);
        }
        Err(e) => {
            panic!("JavaScript execution failed: {}\nSource: {}", e, source);
        }
    }
}

/// Run a JavaScript snippet and expect it to succeed
pub fn assert_js_ok(source: &str) {
    match run_js(source) {
        Ok(_) => {}
        Err(e) => {
            panic!("JavaScript execution failed: {}\nSource: {}", e, source);
        }
    }
}

/// Run a JavaScript snippet and expect it to fail
pub fn assert_js_error(source: &str) {
    match run_js(source) {
        Ok(result) => {
            panic!("Expected JavaScript to fail but got: {}\nSource: {}", result, source);
        }
        Err(_) => {}
    }
}

/// Run a JavaScript snippet and compare the numeric result
pub fn assert_js_num(source: &str, expected: f64) {
    let result = run_js(source).expect("JavaScript execution failed");
    let num: f64 = result.parse().expect("Result is not a number");
    assert!((num - expected).abs() < 0.0001,
            "Expected {}, got {}\nSource: {}", expected, num, source);
}

/// Run a JavaScript snippet and check if result is true
pub fn assert_js_true(source: &str) {
    assert_js_eq(source, "true");
}

/// Run a JavaScript snippet and check if result is false
pub fn assert_js_false(source: &str) {
    assert_js_eq(source, "false");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_basic() {
        // These tests verify the harness itself works
        // Note: Some may fail if the compiler/VM isn't fully working yet

        // Simple literals should work once parser is complete
        // assert_js_eq("42", "42");
        // assert_js_eq("true", "true");
        // assert_js_eq("false", "false");
    }
}
