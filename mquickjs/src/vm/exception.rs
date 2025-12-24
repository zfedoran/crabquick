//! Exception handling

use crate::value::JSValue;

/// Exception state
pub struct Exception {
    // TODO: Implement fields:
    // - value: JSValue (exception value)
    // - stack_trace: Vec<StackFrame>
    _placeholder: u8,
}

impl Exception {
    /// Creates a new exception
    pub fn new(_value: JSValue) -> Self {
        Exception {
            _placeholder: 0,
        }
    }

    /// Returns the exception value
    pub fn value(&self) -> JSValue {
        // TODO: Return value field
        JSValue::undefined()
    }

    /// Captures the current stack trace
    pub fn capture_stack_trace(&mut self) {
        // TODO: Walk call frames
        // TODO: Build stack trace
    }
}
