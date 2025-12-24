//! JavaScript execution context
//!
//! The Context is the main entry point for interacting with the JavaScript engine.
//! It manages memory, the runtime environment, and provides the API for evaluating
//! JavaScript code.

use crate::memory::{Arena, GarbageCollector, HeapIndex};
use crate::value::JSValue;

/// JavaScript execution context
///
/// Manages memory allocation, garbage collection, and the runtime environment.
/// All JavaScript operations must be performed through a Context.
///
/// # Example
///
/// ```rust,ignore
/// use mquickjs::Context;
///
/// let mut ctx = Context::new(8192);
/// let result = ctx.eval("1 + 1", "script.js", 0)?;
/// ```
pub struct Context {
    /// Memory arena for heap allocations
    arena: Arena,
    /// Garbage collector state
    gc: GarbageCollector,
    /// Global object (null until initialized)
    global_object: JSValue,
    /// Current exception value (if any)
    exception_value: JSValue,
    // TODO: Add more fields:
    // - unique_strings: SortedArray<JSString>
    // - class_array: Vec<JSClass>
    // - interrupt_handler: Option<InterruptHandler>
}

impl Context {
    /// Creates a new JavaScript context with the specified memory size
    ///
    /// # Arguments
    ///
    /// * `memory_size` - Size of the heap in bytes
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = Context::new(8192); // 8 KB heap
    /// ```
    pub fn new(memory_size: usize) -> Self {
        Context {
            arena: Arena::new(memory_size),
            gc: GarbageCollector::new(),
            global_object: JSValue::null(),
            exception_value: JSValue::undefined(),
        }
        // TODO: Initialize global object and built-ins
    }

    /// Evaluates JavaScript source code
    ///
    /// # Arguments
    ///
    /// * `source` - JavaScript source code
    /// * `filename` - Filename for error reporting
    /// * `eval_flags` - Evaluation flags
    ///
    /// # Returns
    ///
    /// The result of evaluating the script, or an exception value
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = ctx.eval("2 + 2", "calc.js", 0)?;
    /// ```
    pub fn eval(&mut self, _source: &str, _filename: &str, _eval_flags: i32) -> JSValue {
        // TODO: Compile and execute source code
        JSValue::undefined()
    }

    /// Triggers garbage collection
    pub fn gc(&mut self) {
        self.gc.collect(&mut self.arena);
    }

    /// Returns the current memory usage in bytes
    #[inline]
    pub fn memory_usage(&self) -> usize {
        self.arena.heap_usage()
    }

    /// Returns the total arena size in bytes
    #[inline]
    pub fn arena_size(&self) -> usize {
        self.arena.size()
    }

    /// Returns the amount of free memory in bytes
    #[inline]
    pub fn free_memory(&self) -> usize {
        self.arena.free_space()
    }

    /// Adds a GC root to protect a value from garbage collection
    pub fn add_root(&mut self, value: JSValue) {
        self.gc.add_root(value);
    }

    /// Removes a GC root
    pub fn remove_root(&mut self, value: JSValue) {
        self.gc.remove_root(value);
    }

    /// Allocates memory from the arena
    ///
    /// This is a low-level method for internal use.
    ///
    /// # Safety
    ///
    /// The caller must initialize the allocated memory properly.
    pub(crate) unsafe fn alloc_raw(
        &mut self,
        size: usize,
        mtag: crate::memory::MemTag,
    ) -> Result<HeapIndex, crate::memory::allocator::OutOfMemory> {
        self.arena.alloc(size, mtag)
    }

    /// Gets a reference to the arena (for internal use)
    #[inline]
    pub(crate) fn arena(&self) -> &Arena {
        &self.arena
    }

    /// Gets a mutable reference to the arena (for internal use)
    #[inline]
    pub(crate) fn arena_mut(&mut self) -> &mut Arena {
        &mut self.arena
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // Arena and GC will be dropped automatically
        // TODO: Call finalizers on remaining objects if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = Context::new(1024);
        assert_eq!(ctx.memory_usage(), 0);
        assert_eq!(ctx.arena_size(), 1024);
        assert_eq!(ctx.free_memory(), 1024);
    }

    #[test]
    fn test_context_gc() {
        let mut ctx = Context::new(2048);

        // Allocate some memory
        let idx1 = unsafe {
            ctx.alloc_raw(64, crate::memory::MemTag::Object).unwrap()
        };

        let val1 = JSValue::from_ptr(idx1);
        ctx.add_root(val1);

        // Allocate more
        let _idx2 = unsafe {
            ctx.alloc_raw(128, crate::memory::MemTag::String).unwrap()
        };

        let usage_before_gc = ctx.memory_usage();
        assert!(usage_before_gc > 0);

        // Run GC
        ctx.gc();

        // Memory usage should still be > 0 because we have a root
        let usage_after_gc = ctx.memory_usage();
        assert!(usage_after_gc > 0);

        // Clean up
        ctx.remove_root(val1);
    }

    #[test]
    fn test_context_roots() {
        let mut ctx = Context::new(2048);

        let idx = unsafe {
            ctx.alloc_raw(64, crate::memory::MemTag::Object).unwrap()
        };
        let val = JSValue::from_ptr(idx);

        // Add root
        ctx.add_root(val);

        // GC should preserve it
        ctx.gc();

        // Remove root
        ctx.remove_root(val);
    }

    #[test]
    fn test_context_memory_tracking() {
        let mut ctx = Context::new(1024);

        let initial_usage = ctx.memory_usage();
        assert_eq!(initial_usage, 0);

        // Allocate something
        let _idx = unsafe {
            ctx.alloc_raw(32, crate::memory::MemTag::String).unwrap()
        };

        let usage_after_alloc = ctx.memory_usage();
        assert!(usage_after_alloc > 0);
        assert!(usage_after_alloc < 1024);

        let free_space = ctx.free_memory();
        assert_eq!(usage_after_alloc + free_space, 1024);
    }
}
