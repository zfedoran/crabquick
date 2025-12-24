//! JavaScript execution context
//!
//! The Context is the main entry point for interacting with the JavaScript engine.
//! It manages memory, the runtime environment, and provides the API for evaluating
//! JavaScript code.

use crate::memory::{Arena, GarbageCollector, HeapIndex, MemTag};
use crate::value::{JSValue, AtomTable};

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
    /// Atom table for interned strings
    atom_table: AtomTable,
    /// Global object (null until initialized)
    global_object: JSValue,
    /// Current exception value (if any)
    exception_value: JSValue,
    // TODO: Add more fields:
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
            atom_table: AtomTable::new(),
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

    // ========== String Operations ==========

    /// Creates a new JavaScript string from a Rust &str
    ///
    /// The string is allocated on the heap and stored in UTF-8 format.
    pub fn new_string(&mut self, s: &str) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
        use crate::value::{JSString, JSStringHeader};

        let bytes = s.as_bytes();
        let len = bytes.len();

        // Check flags
        let is_ascii = JSString::check_ascii(bytes);
        let is_numeric = JSString::check_numeric(bytes);

        // Calculate total size: MemBlockHeader + JSStringHeader + UTF-8 data
        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + JSString::alloc_size(len);

        // Allocate memory
        let index = unsafe { self.alloc_raw(total_size, MemTag::String)? };

        // Initialize the string header
        unsafe {
            let string: &mut JSString = self.arena.get_mut(index);
            *string.header_mut() = JSStringHeader::new(len, is_ascii, is_numeric);

            // Copy UTF-8 data
            let data_ptr = (string as *mut JSString as *mut u8)
                .add(core::mem::size_of::<JSStringHeader>());
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), data_ptr, len);
        }

        Ok(JSValue::from_ptr(index))
    }

    /// Gets a &str reference to a JavaScript string
    ///
    /// Returns None if the value is not a string.
    pub fn get_string(&self, val: JSValue) -> Option<&str> {
        let index = val.to_ptr()?;

        unsafe {
            // Check memory tag
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::String {
                return None;
            }

            let string: &crate::value::JSString = self.arena.get(index);
            Some(string.as_str())
        }
    }

    /// Creates a new JavaScript number from an f64
    ///
    /// If the value can be represented as an inline integer, returns an inline value.
    /// Otherwise, allocates a boxed Float64 on the heap.
    pub fn new_number(&mut self, value: f64) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
        use crate::value::JSFloat64;

        // Try to inline as integer
        if JSFloat64::can_inline(value) {
            return Ok(JSValue::from_int(value as i32));
        }

        // Allocate boxed float64
        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + JSFloat64::alloc_size();

        let index = unsafe { self.alloc_raw(total_size, MemTag::Float64)? };

        unsafe {
            let float64: &mut JSFloat64 = self.arena.get_mut(index);
            *float64 = JSFloat64::new(value);
        }

        Ok(JSValue::from_ptr(index))
    }

    /// Gets the numeric value of a JSValue
    ///
    /// Returns None if the value is not a number.
    pub fn get_number(&self, val: JSValue) -> Option<f64> {
        // Check if it's an inline integer
        if let Some(i) = val.to_int() {
            return Some(i as f64);
        }

        // Check if it's a boxed float64
        let index = val.to_ptr()?;

        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::Float64 {
                return None;
            }

            let float64: &crate::value::JSFloat64 = self.arena.get(index);
            Some(float64.value())
        }
    }

    // ========== Array Operations ==========

    /// Allocates a JSValueArray with the specified capacity
    ///
    /// The array is initially empty but has space for `capacity` elements.
    pub fn alloc_value_array(&mut self, capacity: usize) -> Result<HeapIndex, crate::memory::allocator::OutOfMemory> {
        use crate::value::{JSValueArray, JSValueArrayHeader};

        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + JSValueArray::alloc_size(capacity);

        let index = unsafe { self.alloc_raw(total_size, MemTag::ValueArray)? };

        unsafe {
            let array: &mut JSValueArray = self.arena.get_mut(index);
            *array.header_mut() = JSValueArrayHeader::new(capacity);

            // Initialize all elements to undefined
            let slice = array.as_full_mut_slice();
            for elem in slice.iter_mut() {
                *elem = JSValue::undefined();
            }
        }

        Ok(index)
    }

    /// Allocates a JSByteArray with the specified capacity
    ///
    /// The array is initially empty but has space for `capacity` bytes.
    pub fn alloc_byte_array(&mut self, capacity: usize) -> Result<HeapIndex, crate::memory::allocator::OutOfMemory> {
        use crate::value::{JSByteArray, JSByteArrayHeader};

        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + JSByteArray::alloc_size(capacity);

        let index = unsafe { self.alloc_raw(total_size, MemTag::ByteArray)? };

        unsafe {
            let array: &mut JSByteArray = self.arena.get_mut(index);
            *array.header_mut() = JSByteArrayHeader::new(capacity);

            // Initialize all bytes to zero
            let slice = array.as_full_mut_slice();
            for byte in slice.iter_mut() {
                *byte = 0;
            }
        }

        Ok(index)
    }

    /// Gets a reference to a value array
    pub fn get_value_array(&self, index: HeapIndex) -> Option<&crate::value::JSValueArray> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::ValueArray {
                return None;
            }
            Some(self.arena.get(index))
        }
    }

    /// Gets a mutable reference to a value array
    pub fn get_value_array_mut(&mut self, index: HeapIndex) -> Option<&mut crate::value::JSValueArray> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::ValueArray {
                return None;
            }
            Some(self.arena.get_mut(index))
        }
    }

    /// Gets a reference to a byte array
    pub fn get_byte_array(&self, index: HeapIndex) -> Option<&crate::value::JSByteArray> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::ByteArray {
                return None;
            }
            Some(self.arena.get(index))
        }
    }

    /// Gets a mutable reference to a byte array
    pub fn get_byte_array_mut(&mut self, index: HeapIndex) -> Option<&mut crate::value::JSByteArray> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::ByteArray {
                return None;
            }
            Some(self.arena.get_mut(index))
        }
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

    #[test]
    fn test_string_creation() {
        let mut ctx = Context::new(2048);

        let val = ctx.new_string("hello").unwrap();
        assert!(val.is_ptr());

        let s = ctx.get_string(val).unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_string_utf8() {
        let mut ctx = Context::new(2048);

        let val = ctx.new_string("你好世界").unwrap();
        let s = ctx.get_string(val).unwrap();
        assert_eq!(s, "你好世界");
    }

    #[test]
    fn test_number_inline() {
        let mut ctx = Context::new(2048);

        let val = ctx.new_number(42.0).unwrap();
        assert!(val.is_int());
        assert_eq!(ctx.get_number(val), Some(42.0));
    }

    #[test]
    fn test_number_boxed() {
        let mut ctx = Context::new(2048);

        let val = ctx.new_number(3.14).unwrap();
        assert!(val.is_ptr());
        assert_eq!(ctx.get_number(val), Some(3.14));
    }

    #[test]
    fn test_value_array() {
        let mut ctx = Context::new(2048);

        let idx = ctx.alloc_value_array(10).unwrap();
        let array = ctx.get_value_array(idx).unwrap();

        assert_eq!(array.header().capacity(), 10);
        assert_eq!(array.header().count(), 0);
    }

    #[test]
    fn test_byte_array() {
        let mut ctx = Context::new(2048);

        let idx = ctx.alloc_byte_array(100).unwrap();
        let array = ctx.get_byte_array(idx).unwrap();

        assert_eq!(array.header().capacity(), 100);
        assert_eq!(array.header().count(), 0);
    }

    #[test]
    fn test_array_push_pop() {
        let mut ctx = Context::new(2048);

        let idx = ctx.alloc_value_array(5).unwrap();

        unsafe {
            let array = ctx.get_value_array_mut(idx).unwrap();

            // Push values
            assert!(array.push(JSValue::from_int(1)));
            assert!(array.push(JSValue::from_int(2)));
            assert!(array.push(JSValue::from_int(3)));

            assert_eq!(array.header().count(), 3);

            // Pop values
            assert_eq!(array.pop(), Some(JSValue::from_int(3)));
            assert_eq!(array.pop(), Some(JSValue::from_int(2)));
            assert_eq!(array.pop(), Some(JSValue::from_int(1)));
            assert_eq!(array.pop(), None);
        }
    }
}
