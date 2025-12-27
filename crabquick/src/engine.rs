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
/// use crabquick::Engine;
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

    #[test]
    fn test_eval_float() {
        let mut engine = Engine::new(8192);

        // Test basic float
        let result = engine.eval("3.14").unwrap();
        let num = engine.context.get_number(result).expect("Should be a number");
        assert!((num - 3.14).abs() < 0.0001);

        // Test larger float
        let result = engine.eval("123.456").unwrap();
        let num = engine.context.get_number(result).expect("Should be a number");
        assert!((num - 123.456).abs() < 0.0001);

        // Test negative float
        let result = engine.eval("-99.99").unwrap();
        let num = engine.context.get_number(result).expect("Should be a number");
        assert!((num - (-99.99)).abs() < 0.0001);
    }

    #[test]
    fn test_eval_large_integer() {
        let mut engine = Engine::new(8192);

        // Test large integers that don't fit in i8 or i16
        let result = engine.eval("12345").unwrap();
        // Large integers may be stored as i32
        if let Some(i) = result.to_int() {
            assert_eq!(i, 12345);
        } else {
            // Or as float
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 12345.0);
        }

        let result = engine.eval("1000000").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 1000000);
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 1000000.0);
        }
    }

    #[test]
    fn test_eval_float_arithmetic() {
        let mut engine = Engine::new(8192);

        // Test simple literal first
        let result = engine.eval("1.5").unwrap();
        let num = engine.context.get_number(result).expect("1.5 should be a number");
        assert!((num - 1.5).abs() < 0.0001, "Expected 1.5, got {}", num);

        // Test float arithmetic
        let result = engine.eval("1.5 + 2.5").unwrap();
        // Check if it's an integer first
        if let Some(i) = result.to_int() {
            assert_eq!(i, 4, "Float arithmetic should return 4 as integer");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert!((num - 4.0).abs() < 0.0001, "Float arithmetic should be 4.0, got {}", num);
        }

        let result = engine.eval("10.0 / 3.0").unwrap();
        let num = engine.context.get_number(result).expect("Should be a number");
        assert!((num - 3.333333).abs() < 0.0001, "Division should be ~3.333, got {}", num);
    }

    #[test]
    fn test_eval_global_assignment() {
        let mut engine = Engine::new(32768); // 16KB heap
        let result = engine.eval("x = 5; x");
        match result {
            Ok(val) => {
                assert_eq!(engine.context.get_number(val), Some(5.0));
            }
            Err(err) => {
                let err_str = engine.value_to_string(err);
                panic!("eval failed with error: {}", err_str);
            }
        }
    }

    #[test]
    fn test_eval_global_multiple() {
        let mut engine = Engine::new(32768); // 16KB heap
        let result = engine.eval("a = 10; b = 20; a + b").unwrap();
        assert_eq!(engine.context.get_number(result), Some(30.0));
    }

    #[test]
    fn test_eval_global_persistence() {
        let mut engine = Engine::new(32768); // 16KB heap

        // Set a global variable
        engine.eval("x = 42").unwrap();

        // Access it in a later eval
        let result = engine.eval("x").unwrap();
        assert_eq!(engine.context.get_number(result), Some(42.0));

        // Modify it
        engine.eval("x = x + 8").unwrap();
        let result = engine.eval("x").unwrap();
        assert_eq!(engine.context.get_number(result), Some(50.0));
    }

    #[test]
    fn test_eval_global_expression_sequence() {
        let mut engine = Engine::new(32768); // 16KB heap

        // Test that global assignments work in expression sequences
        let result = engine.eval("y = 5; z = y * 2; z + 3").unwrap();
        assert_eq!(engine.context.get_number(result), Some(13.0));
    }

    #[test]
    fn test_eval_math_abs() {
        let mut engine = Engine::new(32768); // Need more memory for builtins
        let result = engine.eval("Math.abs(-5)");
        match result {
            Ok(val) => assert_eq!(engine.context.get_number(val), Some(5.0)),
            Err(err) => {
                let err_str = engine.value_to_string(err);
                panic!("eval failed: {}", err_str);
            }
        }
    }

    #[test]
    fn test_eval_math_floor() {
        let mut engine = Engine::new(32768);
        let result = engine.eval("Math.floor(3.7)").unwrap();
        assert_eq!(engine.context.get_number(result), Some(3.0));
    }

    #[test]
    fn test_eval_math_ceil() {
        let mut engine = Engine::new(32768);
        let result = engine.eval("Math.ceil(3.2)").unwrap();
        assert_eq!(engine.context.get_number(result), Some(4.0));
    }

    #[test]
    fn test_eval_math_round() {
        let mut engine = Engine::new(32768);
        let result = engine.eval("Math.round(3.5)").unwrap();
        assert_eq!(engine.context.get_number(result), Some(4.0));
    }

    #[test]
    fn test_eval_math_max() {
        let mut engine = Engine::new(32768);
        let result = engine.eval("Math.max(1, 5, 3)").unwrap();
        assert_eq!(engine.context.get_number(result), Some(5.0));
    }

    #[test]
    fn test_eval_math_min() {
        let mut engine = Engine::new(32768);
        let result = engine.eval("Math.min(1, 5, 3)").unwrap();
        assert_eq!(engine.context.get_number(result), Some(1.0));
    }

    #[test]
    fn test_eval_console_log() {
        let mut engine = Engine::new(32768);
        // console.log should return a function
        let result = engine.eval("console.log").unwrap();
        // Should be a function object (pointer)
        assert!(result.is_ptr());
    }

    #[test]
    fn test_eval_math_object() {
        let mut engine = Engine::new(32768);
        // Math should be an object
        let result = engine.eval("Math").unwrap();
        assert!(result.is_ptr());
    }

    #[test]
    fn test_eval_math_abs_property() {
        let mut engine = Engine::new(32768);
        // Math.abs should be a function
        let result = engine.eval("Math.abs").unwrap();
        if result.is_undefined() {
            panic!("Math.abs returned undefined");
        }
        println!("Math.abs result: {:?}", result);
        println!("Is ptr: {}", result.is_ptr());

        assert!(result.is_ptr(), "Math.abs should be a pointer, got: {:?}", result);

        // Check if it's a native function
        if let Some(cfunc) = engine.context.get_native_function(result) {
            println!("Found native function with length: {}", cfunc.length());
        } else {
            panic!("Math.abs should be a native function but got None from get_native_function");
        }
    }

    #[test]
    fn test_function_declaration_simple() {
        let mut engine = Engine::new(8192);

        // Do everything in one eval - function declaration followed by call
        let result = engine.eval("function add(a, b) { return a + b; } add(2, 3)").unwrap();
        println!("Result: {:?}, is_int: {}, is_ptr: {}, is_undef: {}",
            result, result.is_int(), result.is_ptr(), result.is_undefined());
        if let Some(num) = engine.context.get_number(result) {
            println!("As number: {}", num);
        }
        assert_eq!(result.to_int(), Some(5), "Simple function call should return 5");
    }

    #[test]
    fn test_function_declaration_no_params() {
        let mut engine = Engine::new(8192);
        let result = engine.eval("function getFortyTwo() { return 42; } getFortyTwo()").unwrap();
        assert_eq!(result.to_int(), Some(42), "No-param function should return 42");
    }

    #[test]
    fn test_function_one_param() {
        let mut engine = Engine::new(8192);
        let result = engine.eval("function double(x) { return x * 2; } double(21)").unwrap();
        assert_eq!(result.to_int(), Some(42), "Double function should return 42");
    }

    #[test]
    fn test_function_with_local_var() {
        let mut engine = Engine::new(8192);
        let result = engine.eval("function sum(a, b) { var result = a + b; return result; } sum(5, 7)").unwrap();
        assert_eq!(result.to_int(), Some(12), "Function with local var should return 12");
    }

    #[test]
    fn test_function_recursive_factorial() {
        let mut engine = Engine::new(8192);
        let result = engine.eval("function factorial(n) { if (n <= 1) return 1; return n * factorial(n - 1); } factorial(5)").unwrap();
        assert_eq!(result.to_int(), Some(120), "Factorial(5) should return 120");
    }

    #[test]
    fn test_function_recursive_fibonacci() {
        let mut engine = Engine::new(8192);
        let result = engine.eval("function fib(n) { if (n <= 1) return n; return fib(n - 1) + fib(n - 2); } fib(10)").unwrap();
        assert_eq!(result.to_int(), Some(55), "Fibonacci(10) should return 55");
    }

    // ========== Type Coercion Tests ==========

    #[test]
    fn test_string_plus_number_concatenation() {
        let mut engine = Engine::new(32768);
        // "5" + 3 should be "53" (string concatenation)
        let result = engine.eval_as_string("\"5\" + 3").unwrap();
        assert_eq!(result, "53", "String + number should concatenate");
    }

    #[test]
    fn test_number_plus_string_concatenation() {
        let mut engine = Engine::new(32768);
        // 5 + "3" should be "53" (string concatenation)
        let result = engine.eval_as_string("5 + \"3\"").unwrap();
        assert_eq!(result, "53", "Number + string should concatenate");
    }

    #[test]
    fn test_string_minus_number() {
        let mut engine = Engine::new(32768);
        // "5" - 3 should be 2 (numeric subtraction)
        let result = engine.eval("\"5\" - 3").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 2, "String - number should be numeric");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 2.0, "String - number should be numeric");
        }
    }

    #[test]
    fn test_string_multiply_number() {
        let mut engine = Engine::new(32768);
        // "5" * 3 should be 15 (numeric multiplication)
        let result = engine.eval("\"5\" * 3").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 15, "String * number should be numeric");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 15.0, "String * number should be numeric");
        }
    }

    #[test]
    fn test_string_divide_number() {
        let mut engine = Engine::new(32768);
        // "10" / 2 should be 5 (numeric division)
        let result = engine.eval("\"10\" / 2").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 5, "String / number should be numeric");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 5.0, "String / number should be numeric");
        }
    }

    #[test]
    fn test_abstract_equality_number_string() {
        let mut engine = Engine::new(32768);
        // 5 == "5" should be true (abstract equality with coercion)
        let result = engine.eval("5 == \"5\"").unwrap();
        assert_eq!(result.to_bool(), Some(true), "5 == \"5\" should be true");
    }

    #[test]
    fn test_strict_equality_number_string() {
        let mut engine = Engine::new(32768);
        // 5 === "5" should be false (strict equality without coercion)
        let result = engine.eval("5 === \"5\"").unwrap();
        assert_eq!(result.to_bool(), Some(false), "5 === \"5\" should be false");
    }

    #[test]
    fn test_null_equals_undefined() {
        let mut engine = Engine::new(32768);
        // null == undefined should be true
        let result = engine.eval("null == undefined").unwrap();
        assert_eq!(result.to_bool(), Some(true), "null == undefined should be true");
    }

    #[test]
    fn test_null_strict_equals_undefined() {
        let mut engine = Engine::new(32768);
        // null === undefined should be false
        let result = engine.eval("null === undefined").unwrap();
        assert_eq!(result.to_bool(), Some(false), "null === undefined should be false");
    }

    #[test]
    fn test_boolean_to_number_hello() {
        let mut engine = Engine::new(32768);
        // !!"hello" should be true (non-empty string is truthy)
        let result = engine.eval("!!\"hello\"").unwrap();
        assert_eq!(result.to_bool(), Some(true), "!!\"hello\" should be true");
    }

    #[test]
    fn test_boolean_to_number_empty_string() {
        let mut engine = Engine::new(32768);
        // !!"" should be false (empty string is falsy)
        let result = engine.eval("!!\"\"").unwrap();
        assert_eq!(result.to_bool(), Some(false), "!!\"\" should be false");
    }

    #[test]
    fn test_boolean_to_number_zero() {
        let mut engine = Engine::new(32768);
        // !!0 should be false (zero is falsy)
        let result = engine.eval("!!0").unwrap();
        assert_eq!(result.to_bool(), Some(false), "!!0 should be false");
    }

    #[test]
    fn test_boolean_to_number_one() {
        let mut engine = Engine::new(32768);
        // !!1 should be true (non-zero is truthy)
        let result = engine.eval("!!1").unwrap();
        assert_eq!(result.to_bool(), Some(true), "!!1 should be true");
    }

    #[test]
    fn test_tonumber_null() {
        let mut engine = Engine::new(32768);
        // null should convert to 0 in numeric context
        let result = engine.eval("null + 5").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 5, "null + 5 should be 5");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 5.0, "null + 5 should be 5");
        }
    }

    #[test]
    fn test_tonumber_true() {
        let mut engine = Engine::new(32768);
        // true should convert to 1 in numeric context
        let result = engine.eval("true + 5").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 6, "true + 5 should be 6");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 6.0, "true + 5 should be 6");
        }
    }

    #[test]
    fn test_tonumber_false() {
        let mut engine = Engine::new(32768);
        // false should convert to 0 in numeric context
        let result = engine.eval("false + 5").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 5, "false + 5 should be 5");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 5.0, "false + 5 should be 5");
        }
    }

    #[test]
    fn test_tostring_number() {
        let mut engine = Engine::new(32768);
        // Number should convert to string in string context
        let result = engine.eval_as_string("\"value: \" + 42").unwrap();
        assert_eq!(result, "value: 42", "Number should convert to string");
    }

    #[test]
    fn test_tostring_true() {
        let mut engine = Engine::new(32768);
        // true should convert to "true"
        let result = engine.eval_as_string("\"boolean: \" + true").unwrap();
        assert_eq!(result, "boolean: true", "true should convert to \"true\"");
    }

    #[test]
    fn test_tostring_false() {
        let mut engine = Engine::new(32768);
        // false should convert to "false"
        let result = engine.eval_as_string("\"boolean: \" + false").unwrap();
        assert_eq!(result, "boolean: false", "false should convert to \"false\"");
    }

    #[test]
    fn test_tostring_null() {
        let mut engine = Engine::new(32768);
        // null should convert to "null"
        let result = engine.eval_as_string("\"value: \" + null").unwrap();
        assert_eq!(result, "value: null", "null should convert to \"null\"");
    }

    #[test]
    fn test_tostring_undefined() {
        let mut engine = Engine::new(32768);
        // undefined should convert to "undefined"
        let result = engine.eval_as_string("\"value: \" + undefined").unwrap();
        assert_eq!(result, "value: undefined", "undefined should convert to \"undefined\"");
    }

    #[test]
    fn test_empty_string_to_number() {
        let mut engine = Engine::new(32768);
        // Empty string should convert to 0
        let result = engine.eval("\"\" - 0").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 0, "Empty string should convert to 0");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 0.0, "Empty string should convert to 0");
        }
    }

    #[test]
    fn test_invalid_string_to_nan() {
        let mut engine = Engine::new(32768);
        // Invalid numeric string should convert to NaN
        let result = engine.eval("\"abc\" - 0").unwrap();
        let num = engine.context.get_number(result).expect("Should be a number");
        assert!(num.is_nan(), "Invalid string should convert to NaN");
    }

    // ========== Critical Bug Tests ==========
    // These tests cover bugs discovered during the examples review.
    // Tests marked with #[ignore] are expected to fail until the bugs are fixed.

    // ---------- Bug 1: Array Indexing Returns Wrong Values ----------

    #[test]
    #[ignore] // Bug: Array indexing returns the index instead of the value
    fn test_array_indexing_first_element() {
        let mut engine = Engine::new(32768);
        // [10, 20, 30][0] should return 10, not 0
        let result = engine.eval("[10, 20, 30][0]").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 10, "Array indexing [0] should return 10");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 10.0, "Array indexing [0] should return 10");
        }
    }

    #[test]
    #[ignore] // Bug: Array indexing returns the index instead of the value
    fn test_array_indexing_middle_element() {
        let mut engine = Engine::new(32768);
        // [10, 20, 30][1] should return 20, not 1
        let result = engine.eval("[10, 20, 30][1]").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 20, "Array indexing [1] should return 20");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 20.0, "Array indexing [1] should return 20");
        }
    }

    #[test]
    #[ignore] // Bug: Array indexing returns the index instead of the value
    fn test_array_indexing_last_element() {
        let mut engine = Engine::new(32768);
        // [10, 20, 30][2] should return 30, not 2
        let result = engine.eval("[10, 20, 30][2]").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 30, "Array indexing [2] should return 30");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 30.0, "Array indexing [2] should return 30");
        }
    }

    #[test]
    #[ignore] // Bug: Array indexing returns the index instead of the value
    fn test_array_indexing_with_variable() {
        let mut engine = Engine::new(32768);
        // Test that indexing with a variable also works correctly
        let result = engine.eval("var arr = [5, 10, 15]; arr[1]").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 10, "Variable array indexing should return 10");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 10.0, "Variable array indexing should return 10");
        }
    }

    #[test]
    #[ignore] // Bug: Array indexing returns the index instead of the value
    fn test_array_indexing_expression_index() {
        let mut engine = Engine::new(32768);
        // Test indexing with an expression
        let result = engine.eval("[100, 200, 300][1 + 1]").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 300, "Array indexing with expression should return 300");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 300.0, "Array indexing with expression should return 300");
        }
    }

    // ---------- Bug 2: For-Loop Stack Underflow ----------

    #[test]
    #[ignore] // Bug: For-loop with assignment update causes stack underflow
    fn test_for_loop_basic_assignment_update() {
        let mut engine = Engine::new(32768);
        // Basic for-loop with assignment update: for (var i = 0; i < 3; i = i + 1)
        let result = engine.eval(
            "var sum = 0; for (var i = 0; i < 3; i = i + 1) { sum = sum + i; } sum"
        ).unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 3, "For-loop should sum 0+1+2 = 3");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 3.0, "For-loop should sum 0+1+2 = 3");
        }
    }

    #[test]
    #[ignore] // Bug: For-loop with assignment update causes stack underflow
    fn test_for_loop_count_iterations() {
        let mut engine = Engine::new(32768);
        // Test that the loop executes the correct number of times
        let result = engine.eval(
            "var count = 0; for (var i = 0; i < 5; i = i + 1) { count = count + 1; } count"
        ).unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 5, "For-loop should iterate 5 times");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 5.0, "For-loop should iterate 5 times");
        }
    }

    #[test]
    #[ignore] // Bug: For-loop with assignment update causes stack underflow
    fn test_for_loop_accumulator_pattern() {
        let mut engine = Engine::new(32768);
        // Test accumulator pattern in for-loop
        let result = engine.eval(
            "var total = 0; for (var i = 1; i < 6; i = i + 1) { total = total + i; } total"
        ).unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 15, "For-loop should sum 1+2+3+4+5 = 15");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 15.0, "For-loop should sum 1+2+3+4+5 = 15");
        }
    }

    #[test]
    #[ignore] // Bug: For-loop with assignment update causes stack underflow
    fn test_for_loop_no_initialization() {
        let mut engine = Engine::new(32768);
        // Test for-loop without initialization
        let result = engine.eval(
            "var i = 0; var sum = 0; for (; i < 4; i = i + 1) { sum = sum + 1; } sum"
        ).unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 4, "For-loop without init should iterate 4 times");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 4.0, "For-loop without init should iterate 4 times");
        }
    }

    // ---------- Bug 3: Increment/Decrement Operators ----------

    #[test]
    #[ignore] // Bug: Postfix increment not implemented
    fn test_postfix_increment_returns_old_value() {
        let mut engine = Engine::new(32768);
        // i++ should return the old value and then increment
        let result = engine.eval("var i = 5; i++").unwrap();
        if let Some(val) = result.to_int() {
            assert_eq!(val, 5, "Postfix increment should return old value (5)");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 5.0, "Postfix increment should return old value (5)");
        }
    }

    #[test]
    #[ignore] // Bug: Postfix increment not implemented
    fn test_postfix_increment_updates_variable() {
        let mut engine = Engine::new(32768);
        // After i++, the variable should be incremented
        let result = engine.eval("var i = 5; i++; i").unwrap();
        if let Some(val) = result.to_int() {
            assert_eq!(val, 6, "Variable should be incremented to 6");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 6.0, "Variable should be incremented to 6");
        }
    }

    #[test]
    #[ignore] // Bug: Prefix increment not implemented
    fn test_prefix_increment_returns_new_value() {
        let mut engine = Engine::new(32768);
        // ++i should increment and return the new value
        let result = engine.eval("var i = 5; ++i").unwrap();
        if let Some(val) = result.to_int() {
            assert_eq!(val, 6, "Prefix increment should return new value (6)");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 6.0, "Prefix increment should return new value (6)");
        }
    }

    #[test]
    #[ignore] // Bug: Prefix increment not implemented
    fn test_prefix_increment_updates_variable() {
        let mut engine = Engine::new(32768);
        // After ++i, the variable should be incremented
        let result = engine.eval("var i = 5; ++i; i").unwrap();
        if let Some(val) = result.to_int() {
            assert_eq!(val, 6, "Variable should be incremented to 6");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 6.0, "Variable should be incremented to 6");
        }
    }

    #[test]
    #[ignore] // Bug: Postfix decrement not implemented
    fn test_postfix_decrement_returns_old_value() {
        let mut engine = Engine::new(32768);
        // i-- should return the old value and then decrement
        let result = engine.eval("var i = 10; i--").unwrap();
        if let Some(val) = result.to_int() {
            assert_eq!(val, 10, "Postfix decrement should return old value (10)");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 10.0, "Postfix decrement should return old value (10)");
        }
    }

    #[test]
    #[ignore] // Bug: Postfix decrement not implemented
    fn test_postfix_decrement_updates_variable() {
        let mut engine = Engine::new(32768);
        // After i--, the variable should be decremented
        let result = engine.eval("var i = 10; i--; i").unwrap();
        if let Some(val) = result.to_int() {
            assert_eq!(val, 9, "Variable should be decremented to 9");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 9.0, "Variable should be decremented to 9");
        }
    }

    #[test]
    #[ignore] // Bug: Prefix decrement not implemented
    fn test_prefix_decrement_returns_new_value() {
        let mut engine = Engine::new(32768);
        // --i should decrement and return the new value
        let result = engine.eval("var i = 10; --i").unwrap();
        if let Some(val) = result.to_int() {
            assert_eq!(val, 9, "Prefix decrement should return new value (9)");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 9.0, "Prefix decrement should return new value (9)");
        }
    }

    #[test]
    #[ignore] // Bug: Prefix decrement not implemented
    fn test_prefix_decrement_updates_variable() {
        let mut engine = Engine::new(32768);
        // After --i, the variable should be decremented
        let result = engine.eval("var i = 10; --i; i").unwrap();
        if let Some(val) = result.to_int() {
            assert_eq!(val, 9, "Variable should be decremented to 9");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 9.0, "Variable should be decremented to 9");
        }
    }

    #[test]
    #[ignore] // Bug: Increment/decrement operators not implemented
    fn test_increment_in_for_loop() {
        let mut engine = Engine::new(32768);
        // Test using i++ in a for-loop (common pattern)
        let result = engine.eval(
            "var sum = 0; for (var i = 0; i < 5; i++) { sum = sum + i; } sum"
        ).unwrap();
        if let Some(val) = result.to_int() {
            assert_eq!(val, 10, "For-loop with i++ should sum 0+1+2+3+4 = 10");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 10.0, "For-loop with i++ should sum 0+1+2+3+4 = 10");
        }
    }

    // ---------- Bug 4: Missing Math Methods ----------

    #[test]
    #[ignore] // Bug: Math.pow not implemented
    fn test_math_pow_basic() {
        let mut engine = Engine::new(32768);
        // Math.pow(2, 8) should return 256
        let result = engine.eval("Math.pow(2, 8)").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 256, "Math.pow(2, 8) should return 256");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 256.0, "Math.pow(2, 8) should return 256");
        }
    }

    #[test]
    #[ignore] // Bug: Math.pow not implemented
    fn test_math_pow_cube() {
        let mut engine = Engine::new(32768);
        // Math.pow(3, 3) should return 27
        let result = engine.eval("Math.pow(3, 3)").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 27, "Math.pow(3, 3) should return 27");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 27.0, "Math.pow(3, 3) should return 27");
        }
    }

    #[test]
    #[ignore] // Bug: Math.pow not implemented
    fn test_math_pow_square() {
        let mut engine = Engine::new(32768);
        // Math.pow(5, 2) should return 25
        let result = engine.eval("Math.pow(5, 2)").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 25, "Math.pow(5, 2) should return 25");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 25.0, "Math.pow(5, 2) should return 25");
        }
    }

    #[test]
    #[ignore] // Bug: Math.sqrt not implemented
    fn test_math_sqrt_basic() {
        let mut engine = Engine::new(32768);
        // Math.sqrt(16) should return 4
        let result = engine.eval("Math.sqrt(16)").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 4, "Math.sqrt(16) should return 4");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 4.0, "Math.sqrt(16) should return 4");
        }
    }

    #[test]
    #[ignore] // Bug: Math.sqrt not implemented
    fn test_math_sqrt_perfect_squares() {
        let mut engine = Engine::new(32768);
        // Math.sqrt(9) should return 3
        let result = engine.eval("Math.sqrt(9)").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 3, "Math.sqrt(9) should return 3");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 3.0, "Math.sqrt(9) should return 3");
        }

        // Math.sqrt(64) should return 8
        let result = engine.eval("Math.sqrt(64)").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 8, "Math.sqrt(64) should return 8");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 8.0, "Math.sqrt(64) should return 8");
        }
    }

    #[test]
    #[ignore] // Bug: Math.sqrt not implemented
    fn test_math_sqrt_non_perfect_square() {
        let mut engine = Engine::new(32768);
        // Math.sqrt(2) should return approximately 1.414
        let result = engine.eval("Math.sqrt(2)").unwrap();
        let num = engine.context.get_number(result).expect("Should be a number");
        assert!((num - 1.414213).abs() < 0.001, "Math.sqrt(2) should be ~1.414");
    }

    // ---------- Bug 5: Array.length Property ----------

    #[test]
    #[ignore] // Bug: Array.length property not implemented
    fn test_array_length_basic() {
        let mut engine = Engine::new(32768);
        // [1, 2, 3].length should return 3
        let result = engine.eval("[1, 2, 3].length").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 3, "Array length should be 3");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 3.0, "Array length should be 3");
        }
    }

    #[test]
    #[ignore] // Bug: Array.length property not implemented
    fn test_array_length_empty() {
        let mut engine = Engine::new(32768);
        // [].length should return 0
        let result = engine.eval("[].length").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 0, "Empty array length should be 0");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 0.0, "Empty array length should be 0");
        }
    }

    #[test]
    #[ignore] // Bug: Array.length property not implemented
    fn test_array_length_large() {
        let mut engine = Engine::new(32768);
        // Test with larger array
        let result = engine.eval("[1, 2, 3, 4, 5, 6, 7, 8, 9, 10].length").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 10, "Array length should be 10");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 10.0, "Array length should be 10");
        }
    }

    #[test]
    #[ignore] // Bug: Array.length property not implemented
    fn test_array_length_variable() {
        let mut engine = Engine::new(32768);
        // Test length on variable-stored array
        let result = engine.eval("var arr = [1, 2, 3, 4]; arr.length").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 4, "Variable array length should be 4");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 4.0, "Variable array length should be 4");
        }
    }

    #[test]
    #[ignore] // Bug: Array.length property not implemented
    fn test_array_length_in_expression() {
        let mut engine = Engine::new(32768);
        // Test using length in an expression
        let result = engine.eval("[1, 2, 3, 4, 5].length * 2").unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 10, "Array length * 2 should be 10");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 10.0, "Array length * 2 should be 10");
        }
    }

    // ---------- Bug 6: Object Method Calls ----------

    #[test]
    #[ignore] // Bug: Object method calls not working
    fn test_object_method_call_basic() {
        let mut engine = Engine::new(32768);
        // Function stored in object property should be callable
        let result = engine.eval(
            "var obj = { method: function() { return 42; } }; obj.method()"
        ).unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 42, "Object method should return 42");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 42.0, "Object method should return 42");
        }
    }

    #[test]
    #[ignore] // Bug: Object method calls not working
    fn test_object_method_with_args() {
        let mut engine = Engine::new(32768);
        // Object method with arguments
        let result = engine.eval(
            "var obj = { add: function(a, b) { return a + b; } }; obj.add(10, 20)"
        ).unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 30, "Object method should return 30");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 30.0, "Object method should return 30");
        }
    }

    #[test]
    #[ignore] // Bug: Object method calls not working
    fn test_object_method_accessing_this() {
        let mut engine = Engine::new(32768);
        // Object method accessing 'this'
        let result = engine.eval(
            "var obj = { value: 100, getValue: function() { return this.value; } }; obj.getValue()"
        ).unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 100, "Object method should access this.value and return 100");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 100.0, "Object method should access this.value and return 100");
        }
    }

    #[test]
    #[ignore] // Bug: Object method calls not working
    fn test_object_method_modifying_this() {
        let mut engine = Engine::new(32768);
        // Object method modifying 'this'
        let result = engine.eval(
            "var obj = { count: 0, increment: function() { this.count = this.count + 1; return this.count; } }; obj.increment()"
        ).unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 1, "Object method should increment and return 1");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 1.0, "Object method should increment and return 1");
        }
    }

    #[test]
    #[ignore] // Bug: Object method calls not working
    fn test_object_multiple_methods() {
        let mut engine = Engine::new(32768);
        // Object with multiple methods
        let result = engine.eval(
            "var calc = { add: function(a, b) { return a + b; }, multiply: function(a, b) { return a * b; } }; calc.add(5, 3) + calc.multiply(2, 4)"
        ).unwrap();
        if let Some(i) = result.to_int() {
            assert_eq!(i, 16, "Should calculate (5+3) + (2*4) = 16");
        } else {
            let num = engine.context.get_number(result).expect("Should be a number");
            assert_eq!(num, 16.0, "Should calculate (5+3) + (2*4) = 16");
        }
    }
}
