//! Function bytecode structure
//!
//! Defines the JSFunctionBytecode type which contains compiled JavaScript functions.

use crate::memory::allocator::HeapIndex;
use crate::value::JSValue;

/// Function bytecode structure
///
/// Represents a compiled JavaScript function with all the metadata needed
/// for execution. This structure is allocated on the heap and referenced
/// by closure objects.
#[repr(C)]
pub struct JSFunctionBytecode {
    /// Function name (for debugging and stack traces)
    pub func_name: JSValue,

    /// Bytecode array (JSByteArray index)
    pub byte_code: HeapIndex,

    /// Constant pool (JSValueArray index)
    pub cpool: HeapIndex,

    /// Variable names (for debugging, JSValueArray index or null)
    pub vars: HeapIndex,

    /// Closure variable names (JSValueArray index or null)
    pub ext_vars: HeapIndex,

    /// Maximum stack size needed
    pub stack_size: u16,

    /// Number of arguments
    pub arg_count: u16,

    /// Source filename (for debugging)
    pub filename: JSValue,

    /// PC-to-line mapping (for stack traces, JSByteArray index or null)
    pub pc2line: HeapIndex,

    /// Flags (reserved for future use)
    pub flags: u32,
}

impl JSFunctionBytecode {
    /// Creates a new function bytecode structure
    pub fn new(
        func_name: JSValue,
        byte_code: HeapIndex,
        cpool: HeapIndex,
        stack_size: u16,
        arg_count: u16,
        filename: JSValue,
    ) -> Self {
        JSFunctionBytecode {
            func_name,
            byte_code,
            cpool,
            vars: HeapIndex::null(),
            ext_vars: HeapIndex::null(),
            stack_size,
            arg_count,
            filename,
            pc2line: HeapIndex::null(),
            flags: 0,
        }
    }

    /// Returns the size of JSFunctionBytecode in bytes
    pub const fn size() -> usize {
        core::mem::size_of::<JSFunctionBytecode>()
    }

    /// Sets the variable names
    pub fn set_vars(&mut self, vars: HeapIndex) {
        self.vars = vars;
    }

    /// Sets the closure variable names
    pub fn set_ext_vars(&mut self, ext_vars: HeapIndex) {
        self.ext_vars = ext_vars;
    }

    /// Sets the PC-to-line mapping
    pub fn set_pc2line(&mut self, pc2line: HeapIndex) {
        self.pc2line = pc2line;
    }

    /// Returns true if this function has closure variables
    pub fn has_closure_vars(&self) -> bool {
        !self.ext_vars.is_null()
    }

    /// Returns true if this function has debug info
    pub fn has_debug_info(&self) -> bool {
        !self.pc2line.is_null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let func_name = JSValue::undefined();
        let byte_code = HeapIndex::from_usize(100);
        let cpool = HeapIndex::from_usize(200);
        let filename = JSValue::undefined();

        let fb = JSFunctionBytecode::new(
            func_name,
            byte_code,
            cpool,
            10,  // stack_size
            2,   // arg_count
            filename,
        );

        assert_eq!(fb.func_name, func_name);
        assert_eq!(fb.byte_code, byte_code);
        assert_eq!(fb.cpool, cpool);
        assert_eq!(fb.stack_size, 10);
        assert_eq!(fb.arg_count, 2);
        assert_eq!(fb.filename, filename);
        assert!(fb.vars.is_null());
        assert!(fb.ext_vars.is_null());
        assert!(fb.pc2line.is_null());
        assert_eq!(fb.flags, 0);
    }

    #[test]
    fn test_setters() {
        let mut fb = JSFunctionBytecode::new(
            JSValue::undefined(),
            HeapIndex::from_usize(100),
            HeapIndex::from_usize(200),
            10,
            2,
            JSValue::undefined(),
        );

        assert!(!fb.has_closure_vars());
        assert!(!fb.has_debug_info());

        fb.set_vars(HeapIndex::from_usize(300));
        fb.set_ext_vars(HeapIndex::from_usize(400));
        fb.set_pc2line(HeapIndex::from_usize(500));

        assert_eq!(fb.vars.as_usize(), 300);
        assert_eq!(fb.ext_vars.as_usize(), 400);
        assert_eq!(fb.pc2line.as_usize(), 500);

        assert!(fb.has_closure_vars());
        assert!(fb.has_debug_info());
    }

    #[test]
    fn test_size() {
        // Ensure size is reasonable (should be around 48-64 bytes depending on platform)
        let size = JSFunctionBytecode::size();
        assert!(size >= 40);
        assert!(size <= 128);
    }
}
