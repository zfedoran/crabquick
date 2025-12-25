//! High-level JavaScript engine API
//!
//! This module provides a simplified interface for executing JavaScript code.
//! It wraps the Context, Compiler, and VM into a single easy-to-use API.

use crate::context::Context;
use crate::value::JSValue;
use crate::compiler;
use crate::runtime;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Memory statistics for the JavaScript engine
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    /// Total heap size in bytes
    pub heap_size: usize,
    /// Heap bytes currently in use
    pub heap_used: usize,
    /// Number of objects allocated (approximate)
    pub object_count: usize,
}

/// High-level JavaScript engine
///
/// The Engine provides a simple API for executing JavaScript code.
/// It manages the execution context, runtime initialization, and provides
/// convenient methods for evaluation and interaction.
///
/// # Example
///
/// ```rust,ignore
/// use mquickjs::Engine;
///
/// let mut engine = Engine::new(65536); // 64 KB heap
/// let result = engine.eval("2 + 3").unwrap();
/// let text = engine.eval_as_string("2 + 3").unwrap();
/// assert_eq!(text, "5");
/// ```
pub struct Engine {
    /// Execution context
    context: Context,
    /// Random state for Math.random()
    random_state: u64,
}

impl Engine {
    /// Create a new JavaScript engine with specified heap size
    ///
    /// # Arguments
    ///
    /// * `heap_size` - Size of the heap in bytes (minimum 1024 bytes recommended)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let engine = Engine::new(65536); // 64 KB heap
    /// ```
    pub fn new(heap_size: usize) -> Self {
        let mut context = Context::new(heap_size);

        // Initialize the runtime (global object, built-ins, etc.)
        // For now we skip this if it fails - we'll improve error handling later
        let _ = runtime::init_runtime(&mut context);

        Engine {
            context,
            random_state: 0x123456789ABCDEF0, // Simple initial seed
        }
    }

    /// Execute JavaScript source code and return the result
    ///
    /// # Arguments
    ///
    /// * `source` - JavaScript source code to execute
    ///
    /// # Returns
    ///
    /// * `Ok(JSValue)` - The result of execution
    /// * `Err(JSValue)` - An exception value (error)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut engine = Engine::new(65536);
    /// let result = engine.eval("1 + 2")?;
    /// ```
    pub fn eval(&mut self, source: &str) -> Result<JSValue, JSValue> {
        // Compile the source code to bytecode
        let bytecode = compiler::compile(source)
            .map_err(|e| self.make_error(&alloc::format!("Compile error: {:?}", e)))?;

        // Store bytecode in a byte array on the heap
        let bytecode_index = self.store_bytecode(&bytecode)
            .map_err(|_| self.make_error("Out of memory storing bytecode"))?;

        // Execute the bytecode
        self.context.execute_bytecode(bytecode_index)
    }

    /// Execute JavaScript and get result as string
    ///
    /// This is a convenience method that calls `eval()` and converts the result
    /// to a string representation.
    ///
    /// # Arguments
    ///
    /// * `source` - JavaScript source code to execute
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - String representation of the result
    /// * `Err(String)` - Error message
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut engine = Engine::new(65536);
    /// let result = engine.eval_as_string("2 + 3")?;
    /// assert_eq!(result, "5");
    /// ```
    pub fn eval_as_string(&mut self, source: &str) -> Result<String, String> {
        match self.eval(source) {
            Ok(value) => Ok(self.value_to_string(value)),
            Err(err) => Err(self.value_to_string(err)),
        }
    }

    /// Get a global variable by name
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the global variable
    ///
    /// # Returns
    ///
    /// The value of the global variable, or None if not found
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let engine = Engine::new(65536);
    /// let undefined = engine.get_global("undefined");
    /// ```
    pub fn get_global(&self, _name: &str) -> Option<JSValue> {
        // TODO: Implement global variable lookup
        // This requires:
        // 1. Getting the global object from context
        // 2. Looking up the property by name
        None
    }

    /// Set a global variable
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the global variable
    /// * `value` - Value to set
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(String)` - Error message
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut engine = Engine::new(65536);
    /// engine.set_global("x", JSValue::from_int(42))?;
    /// ```
    pub fn set_global(&mut self, _name: &str, _value: JSValue) -> Result<(), String> {
        // TODO: Implement global variable setting
        // This requires:
        // 1. Getting the global object from context
        // 2. Setting the property by name
        Ok(())
    }

    /// Call a JavaScript function
    ///
    /// # Arguments
    ///
    /// * `func` - The function to call
    /// * `args` - Arguments to pass to the function
    ///
    /// # Returns
    ///
    /// * `Ok(JSValue)` - The return value
    /// * `Err(JSValue)` - An exception value
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut engine = Engine::new(65536);
    /// engine.eval("function add(a, b) { return a + b; }")?;
    /// let func = engine.get_global("add").unwrap();
    /// let result = engine.call_function(func, &[JSValue::from_int(2), JSValue::from_int(3)])?;
    /// ```
    pub fn call_function(&mut self, func: JSValue, args: &[JSValue]) -> Result<JSValue, JSValue> {
        self.context.call_function(func, JSValue::undefined(), args)
    }

    /// Run garbage collection
    ///
    /// This forces a garbage collection cycle, freeing memory used by
    /// unreachable objects.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut engine = Engine::new(65536);
    /// // ... do some work ...
    /// engine.gc(); // Free unused memory
    /// ```
    pub fn gc(&mut self) {
        self.context.gc();
    }

    /// Get memory statistics
    ///
    /// Returns information about heap usage and object allocation.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let engine = Engine::new(65536);
    /// let stats = engine.memory_stats();
    /// println!("Heap usage: {} / {}", stats.heap_used, stats.heap_size);
    /// ```
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            heap_size: self.context.arena_size(),
            heap_used: self.context.memory_usage(),
            object_count: 0, // TODO: Track object count
        }
    }

    /// Get next random number (for Math.random implementation)
    pub(crate) fn next_random(&mut self) -> f64 {
        // Simple xorshift64 PRNG
        self.random_state ^= self.random_state << 13;
        self.random_state ^= self.random_state >> 7;
        self.random_state ^= self.random_state << 17;

        // Convert to [0, 1) range
        (self.random_state as f64) / (u64::MAX as f64)
    }

    // ========== Helper Methods ==========

    /// Store bytecode in a heap-allocated byte array
    fn store_bytecode(&mut self, bytecode: &[u8]) -> Result<crate::memory::HeapIndex, crate::memory::allocator::OutOfMemory> {
        let len = bytecode.len();
        let index = self.context.alloc_byte_array(len)?;

        unsafe {
            let array = self.context.get_byte_array_mut(index).unwrap();
            let slice = array.as_full_mut_slice();
            slice[..len].copy_from_slice(bytecode);
            array.header_mut().set_count(len);
        }

        Ok(index)
    }

    /// Convert a JSValue to a string representation
    fn value_to_string(&self, value: JSValue) -> String {
        if value.is_undefined() {
            return "undefined".to_string();
        }
        if value.is_null() {
            return "null".to_string();
        }
        if value.is_bool() {
            return if value.to_bool().unwrap() { "true" } else { "false" }.to_string();
        }
        if let Some(i) = value.to_int() {
            return alloc::format!("{}", i);
        }
        if let Some(f) = self.context.get_number(value) {
            return alloc::format!("{}", f);
        }
        if let Some(s) = self.context.get_string(value) {
            return s.to_string();
        }

        // For objects, arrays, functions, etc.
        "[object]".to_string()
    }

    /// Create an error value
    fn make_error(&mut self, message: &str) -> JSValue {
        // Try to create a string error message
        self.context.new_string(message)
            .unwrap_or(JSValue::undefined())
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(65536) // 64 KB default heap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let engine = Engine::new(8192);
        let stats = engine.memory_stats();
        assert_eq!(stats.heap_size, 8192);
    }

    #[test]
    fn test_engine_default() {
        let engine = Engine::default();
        let stats = engine.memory_stats();
        assert_eq!(stats.heap_size, 65536);
    }

    #[test]
    fn test_memory_stats() {
        let mut engine = Engine::new(4096);
        let stats = engine.memory_stats();
        assert_eq!(stats.heap_size, 4096);
        assert!(stats.heap_used <= stats.heap_size);
    }

    #[test]
    fn test_gc() {
        let mut engine = Engine::new(8192);
        // Just ensure GC doesn't crash
        engine.gc();
    }

    #[test]
    fn test_random() {
        let mut engine = Engine::new(1024);
        let r1 = engine.next_random();
        let r2 = engine.next_random();

        assert!(r1 >= 0.0 && r1 < 1.0);
        assert!(r2 >= 0.0 && r2 < 1.0);
        assert_ne!(r1, r2); // Should be different
    }

    #[test]
    fn test_eval_returns_expression_value() {
        let mut engine = Engine::new(8192);

        // Test simple arithmetic - should return 4, not undefined
        let result = engine.eval("2 + 2").unwrap();
        assert_eq!(result.to_int(), Some(4));

        // Test more complex expression
        let result = engine.eval("10 * 5 + 3").unwrap();
        assert_eq!(result.to_int(), Some(53));

        // Test boolean expression
        let result = engine.eval("true").unwrap();
        assert_eq!(result.to_bool(), Some(true));

        // Test string result as string
        let result = engine.eval_as_string("42").unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_eval_multiple_statements() {
        let mut engine = Engine::new(8192);

        // When there are multiple statements, only the last expression should be returned
        let result = engine.eval("1 + 1; 2 + 2").unwrap();
        assert_eq!(result.to_int(), Some(4));

        // Test with a declaration followed by expression
        let result = engine.eval("var x = 10; x * 2").unwrap();
        assert_eq!(result.to_int(), Some(20));
    }

    #[test]
    fn test_eval_non_expression_returns_undefined() {
        let mut engine = Engine::new(8192);

        // Variable declarations should still return undefined
        let result = engine.eval("var x = 5;").unwrap();
        assert!(result.is_undefined());

        // Empty program should return undefined
        let result = engine.eval("").unwrap();
        assert!(result.is_undefined());
    }
}
