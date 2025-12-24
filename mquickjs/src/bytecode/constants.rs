//! Constant pool management

use crate::value::JSValue;

/// Constant pool for bytecode functions
pub struct ConstantPool {
    // TODO: Implement fields:
    // - constants: Vec<JSValue>
    _placeholder: u8,
}

impl ConstantPool {
    /// Creates a new empty constant pool
    pub fn new() -> Self {
        ConstantPool {
            _placeholder: 0,
        }
    }

    /// Adds a constant to the pool
    ///
    /// Returns the index of the constant (new or existing).
    pub fn add(&mut self, _value: JSValue) -> u16 {
        // TODO: Check for duplicate
        // TODO: Add constant
        // TODO: Return index
        0
    }

    /// Gets a constant by index
    pub fn get(&self, _index: u16) -> Option<JSValue> {
        // TODO: Bounds check and return constant
        None
    }

    /// Returns the number of constants
    pub fn len(&self) -> usize {
        // TODO: Return constants.len()
        0
    }

    /// Returns true if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for ConstantPool {
    fn default() -> Self {
        Self::new()
    }
}
