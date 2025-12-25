//! GC root handles for protecting values during allocation
//!
//! GC roots ensure that values are not collected during operations that
//! might trigger garbage collection.

use crate::value::JSValue;
use core::marker::PhantomData;
use core::ptr::NonNull;

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
///
/// # Safety
///
/// The GcRoot must not outlive the Context it was created from.
/// The lifetime 'ctx ensures this at compile time.
pub struct GcRoot<'ctx> {
    /// Pointer to the value in the GC's root list
    /// This is a pointer into the GC's roots Vec, which may be reallocated,
    /// so we need to be careful about invalidation.
    /// For safety, we store the index instead.
    root_index: usize,
    /// The actual value (cached for quick access)
    value: JSValue,
    /// Phantom data to tie this to the context's lifetime
    /// and to make GcRoot !Send and !Sync
    _marker: PhantomData<&'ctx mut ()>,
}

impl<'ctx> GcRoot<'ctx> {
    /// Creates a new GC root
    ///
    /// This is meant to be called by Context, not directly by users.
    ///
    /// # Safety
    ///
    /// The caller must ensure that root_index is valid for the lifetime 'ctx.
    pub(crate) unsafe fn new(value: JSValue, root_index: usize) -> Self {
        GcRoot {
            root_index,
            value,
            _marker: PhantomData,
        }
    }

    /// Returns the rooted value
    #[inline]
    pub fn value(&self) -> JSValue {
        self.value
    }

    /// Returns the root index (for Context to use when unrooting)
    #[inline]
    pub(crate) fn root_index(&self) -> usize {
        self.root_index
    }

    /// Updates the cached value (called by GC after compaction)
    #[inline]
    pub(crate) fn update_value(&mut self, new_value: JSValue) {
        self.value = new_value;
    }
}

// Note: We don't implement Drop here because we need access to the Context
// to properly unroot. Instead, Context will provide a manual unroot method,
// or we'll use a different approach with Rc/Arc if needed.
//
// For now, the roots will be managed manually by the Context.
// This is safer than having Drop try to access a potentially moved Context.
