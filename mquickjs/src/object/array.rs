//! JavaScript array implementation

use crate::value::JSValue;

/// Array class data
pub struct JSArray {
    // TODO: Implement fields:
    // - tab: JSValue (JSValueArray index or null)
    // - len: u32 (array length)
    _placeholder: u8,
}

impl JSArray {
    /// Creates a new array with the specified length
    pub fn new(_length: u32) -> Self {
        // TODO: Allocate backing storage
        JSArray {
            _placeholder: 0,
        }
    }

    /// Returns the array length
    pub fn len(&self) -> u32 {
        // TODO: Return len field
        0
    }

    /// Returns true if the array is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Sets the array length
    pub fn set_len(&mut self, _new_len: u32) {
        // TODO: Resize backing storage if needed
        // TODO: Truncate or extend
    }

    /// Gets an element by index
    pub fn get(&self, _index: u32) -> JSValue {
        // TODO: Bounds check and return element
        JSValue::undefined()
    }

    /// Sets an element by index
    pub fn set(&mut self, _index: u32, _value: JSValue) {
        // TODO: Resize if needed
        // TODO: Set element
    }
}

/// Value array storage (JSValue[])
pub struct JSValueArray {
    // TODO: Implement fields:
    // - size: u32
    // - values: [JSValue] (flexible array member)
    _placeholder: u8,
}

impl JSValueArray {
    /// Creates a new value array
    pub fn new(_size: usize) -> Self {
        JSValueArray {
            _placeholder: 0,
        }
    }

    /// Returns the size
    pub fn len(&self) -> usize {
        // TODO: Return size field
        0
    }

    /// Returns true if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
