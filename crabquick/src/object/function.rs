//! JavaScript function implementation

use crate::value::JSValue;
use crate::context::Context;

/// Native function type
pub type NativeFn = fn(&mut Context, JSValue, &[JSValue]) -> Result<JSValue, JSValue>;

/// Bytecode function object
/// Stores a reference to compiled bytecode and metadata
#[repr(C)]
pub struct JSBytecodeFunction {
    /// Index to the bytecode array in the heap
    pub bytecode_index: crate::memory::HeapIndex,
    /// Number of parameters
    pub param_count: u8,
    /// Number of local variable slots (including parameters)
    pub local_count: u8,
    /// Reserved for future use
    _reserved: u16,
}

impl JSBytecodeFunction {
    /// Creates a new bytecode function
    pub fn new(
        bytecode_index: crate::memory::HeapIndex,
        param_count: u8,
        local_count: u8,
    ) -> Self {
        JSBytecodeFunction {
            bytecode_index,
            param_count,
            local_count,
            _reserved: 0,
        }
    }

    /// Returns the bytecode index
    pub fn bytecode_index(&self) -> crate::memory::HeapIndex {
        self.bytecode_index
    }

    /// Returns the parameter count
    pub fn param_count(&self) -> u8 {
        self.param_count
    }

    /// Returns the local count
    pub fn local_count(&self) -> u8 {
        self.local_count
    }
}

/// Old JSFunction structure - kept for compatibility
pub struct JSFunction {
    // TODO: Implement fields for full QuickJS compatibility if needed
    _placeholder: u8,
}

impl JSFunction {
    /// Creates a new function
    pub fn new() -> Self {
        JSFunction {
            _placeholder: 0,
        }
    }

    /// Returns the function name
    pub fn name(&self) -> JSValue {
        JSValue::undefined()
    }

    /// Returns the argument count
    pub fn arg_count(&self) -> u16 {
        0
    }
}

impl Default for JSFunction {
    fn default() -> Self {
        Self::new()
    }
}

/// Variable reference for closures
///
/// Stored on heap with MemTag::VarRef. Contains a captured variable value
/// that can be shared between the creating function and any closures that
/// capture it.
#[repr(C)]
pub struct JSVarRef {
    /// The captured value (always detached - stored on heap)
    value: JSValue,
}

impl JSVarRef {
    /// Creates a new variable reference with the given value
    pub fn new(value: JSValue) -> Self {
        JSVarRef { value }
    }

    /// Returns the captured value
    pub fn value(&self) -> JSValue {
        self.value
    }

    /// Sets the captured value
    pub fn set_value(&mut self, value: JSValue) {
        self.value = value;
    }
}

impl Default for JSVarRef {
    fn default() -> Self {
        Self::new(JSValue::undefined())
    }
}

/// Closure object - a function plus its captured environment
///
/// Stored on heap with MemTag::ClosureData. Contains a reference to the
/// underlying bytecode function plus an array of HeapIndex values pointing
/// to JSVarRef objects for each captured variable.
#[repr(C)]
pub struct JSClosure {
    /// HeapIndex pointing to the function's bytecode (not a function table index!)
    pub bytecode_index: crate::memory::HeapIndex,
    /// Number of parameters
    pub param_count: u8,
    /// Number of local variables (including parameters)
    pub local_count: u8,
    /// Number of captured variables
    pub var_ref_count: u8,
    /// For named function expressions: slot index where the function should be stored
    /// 0xFF means no self-reference needed
    pub self_name_slot: u8,
    // Followed by: [HeapIndex; var_ref_count] - the var_refs array
}

impl JSClosure {
    /// Returns the size needed for a closure with N var refs
    pub fn alloc_size(var_ref_count: usize) -> usize {
        core::mem::size_of::<JSClosure>()
            + var_ref_count * core::mem::size_of::<crate::memory::HeapIndex>()
    }

    /// Gets a pointer to the var_refs array
    ///
    /// # Safety
    /// Caller must ensure the closure was allocated with enough space
    pub unsafe fn var_refs_ptr(&self) -> *const crate::memory::HeapIndex {
        (self as *const Self).add(1) as *const crate::memory::HeapIndex
    }

    /// Gets a mutable pointer to the var_refs array
    ///
    /// # Safety
    /// Caller must ensure the closure was allocated with enough space
    pub unsafe fn var_refs_ptr_mut(&mut self) -> *mut crate::memory::HeapIndex {
        (self as *mut Self).add(1) as *mut crate::memory::HeapIndex
    }

    /// Gets the HeapIndex for a captured variable
    pub fn get_var_ref(&self, idx: usize) -> crate::memory::HeapIndex {
        assert!(idx < self.var_ref_count as usize);
        unsafe {
            *self.var_refs_ptr().add(idx)
        }
    }

    /// Sets the HeapIndex for a captured variable
    pub fn set_var_ref(&mut self, idx: usize, heap_idx: crate::memory::HeapIndex) {
        assert!(idx < self.var_ref_count as usize);
        unsafe {
            *self.var_refs_ptr_mut().add(idx) = heap_idx;
        }
    }
}

impl Default for JSClosure {
    fn default() -> Self {
        JSClosure {
            bytecode_index: crate::memory::HeapIndex(0),
            param_count: 0,
            local_count: 0,
            var_ref_count: 0,
            self_name_slot: 0xFF,  // 0xFF means no self-reference
        }
    }
}

/// C function data
pub struct JSCFunction {
    /// Native function pointer
    pub func_ptr: NativeFn,
    /// Argument count (for Function.length)
    pub length: u16,
}

impl JSCFunction {
    /// Creates a new C function
    pub fn new(func_ptr: NativeFn, length: u16) -> Self {
        JSCFunction {
            func_ptr,
            length,
        }
    }

    /// Gets the function pointer
    pub fn func_ptr(&self) -> NativeFn {
        self.func_ptr
    }

    /// Gets the argument count
    pub fn length(&self) -> u16 {
        self.length
    }
}

// Note: No Default implementation for JSCFunction since it requires a function pointer
