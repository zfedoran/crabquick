//! JavaScript function implementation

use crate::value::JSValue;

/// Function bytecode
pub struct JSFunction {
    // TODO: Implement fields:
    // - func_name: JSValue
    // - byte_code: JSValue (JSByteArray index)
    // - cpool: JSValue (constant pool - JSValueArray)
    // - vars: JSValue (variable names)
    // - ext_vars: JSValue (external variables for closures)
    // - stack_size: u16
    // - arg_count: u16
    // - filename: JSValue
    // - pc2line: JSValue (debug info)
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
        // TODO: Return func_name field
        JSValue::undefined()
    }

    /// Returns the argument count
    pub fn arg_count(&self) -> u16 {
        // TODO: Return arg_count field
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
    // TODO: Implement fields:
    // - func_ptr: usize (index into C function table)
    // - length: u16 (argument count)
    _placeholder: u8,
}

impl JSCFunction {
    /// Creates a new C function
    pub fn new() -> Self {
        JSCFunction {
            _placeholder: 0,
        }
    }
}

impl Default for JSCFunction {
    fn default() -> Self {
        Self::new()
    }
}
