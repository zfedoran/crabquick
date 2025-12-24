//! Basic integration tests

use mquickjs::{Context, JSValue};

#[test]
fn test_context_creation() {
    let _ctx = Context::new(8192);
}

#[test]
fn test_value_int() {
    let val = JSValue::from_int(42);
    assert_eq!(val.to_int(), Some(42));
}

#[test]
fn test_value_special() {
    let null = JSValue::null();
    assert!(null.is_null());

    let undef = JSValue::undefined();
    assert!(undef.is_undefined());

    let t = JSValue::bool(true);
    assert!(t.is_bool());
    assert_eq!(t.to_bool(), Some(true));

    let f = JSValue::bool(false);
    assert_eq!(f.to_bool(), Some(false));
}

#[test]
fn test_eval_stub() {
    let mut ctx = Context::new(8192);
    let result = ctx.eval("2 + 2", "test.js", 0);
    // For now, eval just returns undefined
    assert!(result.is_undefined());
}
