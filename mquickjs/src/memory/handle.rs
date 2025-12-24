//! GC root handles for protecting values during allocation
//!
//! GC roots ensure that values are not collected during operations that
//! might trigger garbage collection.

use crate::value::JSValue;
use core::marker::PhantomData;

/// RAII handle for a GC root
///
/// Automatically unroots the value when dropped.
///
/// # Example
///
/// ```rust,ignore
/// let obj = ctx.root(ctx.new_object());
/// ctx.new_string("might trigger GC"); // obj is protected
/// let val = obj.value(); // still valid
/// // obj is automatically unrooted when it goes out of scope
/// ```
pub struct GcRoot<'ctx> {
    // TODO: Implement fields:
    // - ctx: &'ctx Context
    // - slot: *mut JSValue
    _marker: PhantomData<&'ctx ()>,
}

impl<'ctx> GcRoot<'ctx> {
    /// Creates a new GC root
    pub fn new(_value: JSValue) -> Self {
        // TODO: Push value onto GC root stack
        GcRoot {
            _marker: PhantomData,
        }
    }

    /// Returns the rooted value
    pub fn value(&self) -> JSValue {
        // TODO: Return value from slot
        JSValue::undefined()
    }
}

impl<'ctx> Drop for GcRoot<'ctx> {
    fn drop(&mut self) {
        // TODO: Pop value from GC root stack
    }
}
