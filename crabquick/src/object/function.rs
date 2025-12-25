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

/// Closure data
pub struct JSClosure {
    // TODO: Implement fields:
    // - func_bytecode: JSValue (JSFunction index)
    // - var_refs: HeapIndex (array of JSVarRef)
    _placeholder: u8,
}

impl JSClosure {
    /// Creates a new closure
    pub fn new() -> Self {
        JSClosure {
            _placeholder: 0,
        }
    }
}

impl Default for JSClosure {
    fn default() -> Self {
        Self::new()
    }
}

/// Variable reference for closures
pub struct JSVarRef {
    // TODO: Implement fields:
    // - is_detached: bool
    // - value: JSValue (if detached)
    // - pvalue: *mut JSValue (if attached to stack)
    _placeholder: u8,
}

impl JSVarRef {
    /// Creates a new variable reference
    pub fn new() -> Self {
        JSVarRef {
            _placeholder: 0,
        }
    }

    /// Returns the value
    pub fn value(&self) -> JSValue {
        // TODO: Return value or *pvalue
        JSValue::undefined()
    }

    /// Sets the value
    pub fn set_value(&mut self, _value: JSValue) {
        // TODO: Set value or *pvalue
    }
}

impl Default for JSVarRef {
    fn default() -> Self {
        Self::new()
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
