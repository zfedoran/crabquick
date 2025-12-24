//! VM stack management

use crate::value::JSValue;

/// Virtual machine stack
pub struct Stack {
    // TODO: Implement fields:
    // - values: Vec<JSValue>
    // - sp: usize (stack pointer)
    _placeholder: u8,
}

impl Stack {
    /// Creates a new stack
    pub fn new(_size: usize) -> Self {
        Stack {
            _placeholder: 0,
        }
    }

    /// Pushes a value onto the stack
    pub fn push(&mut self, _value: JSValue) {
        // TODO: Check overflow
        // TODO: Push value
    }

    /// Pops a value from the stack
    pub fn pop(&mut self) -> Option<JSValue> {
        // TODO: Check underflow
        // TODO: Pop value
        None
    }

    /// Gets a value at an offset from the stack pointer
    pub fn get(&self, _offset: isize) -> Option<JSValue> {
        // TODO: Calculate index
        // TODO: Bounds check
        None
    }

    /// Sets a value at an offset from the stack pointer
    pub fn set(&mut self, _offset: isize, _value: JSValue) {
        // TODO: Calculate index
        // TODO: Bounds check
        // TODO: Set value
    }
}
