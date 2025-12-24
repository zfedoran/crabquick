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

#[test]
fn test_memory_tracking() {
    let ctx = Context::new(8192);
    assert_eq!(ctx.memory_usage(), 0);
    assert_eq!(ctx.arena_size(), 8192);
    assert_eq!(ctx.free_memory(), 8192);
}

#[test]
fn test_gc_basic() {
    let mut ctx = Context::new(8192);

    // Allocate some memory
    unsafe {
        let _ = ctx.alloc_raw(64, mquickjs::memory::MemTag::Object).unwrap();
        let _ = ctx.alloc_raw(128, mquickjs::memory::MemTag::String).unwrap();
    }

    let usage_before = ctx.memory_usage();
    assert!(usage_before > 0);

    // Run GC (nothing is rooted, but compaction is not fully implemented yet)
    ctx.gc();

    // Memory is still allocated
    let usage_after = ctx.memory_usage();
    assert!(usage_after > 0);
}

#[test]
fn test_gc_roots() {
    use mquickjs::memory::HeapIndex;

    let mut ctx = Context::new(8192);

    // Allocate and root an object
    let idx = unsafe {
        ctx.alloc_raw(64, mquickjs::memory::MemTag::Object).unwrap()
    };

    let val = JSValue::from_ptr(idx);
    ctx.add_root(val);

    // Run GC
    ctx.gc();

    // Remove root
    ctx.remove_root(val);
}

#[test]
fn test_value_ptr_roundtrip() {
    use mquickjs::memory::HeapIndex;

    let mut ctx = Context::new(8192);

    let idx = unsafe {
        ctx.alloc_raw(64, mquickjs::memory::MemTag::Object).unwrap()
    };

    let val = JSValue::from_ptr(idx);
    assert!(val.is_ptr());
    assert_eq!(val.to_ptr(), Some(idx));
}
