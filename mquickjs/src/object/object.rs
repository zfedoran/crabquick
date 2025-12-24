//! JavaScript object representation

use crate::value::JSValue;

/// JavaScript object
///
/// Layout:
/// - Header with class_id and GC bits
/// - Prototype reference
/// - Properties reference
/// - Class-specific data reference
#[repr(C)]
pub struct JSObject {
    // TODO: Implement fields:
    // - header: u32 (packed: gc_mark, class_id, flags)
    // - proto: JSValue
    // - props: JSValue (PropertyTable index)
    // - class_data: HeapIndex
    _placeholder: u8,
}

impl JSObject {
    /// Creates a new empty object
    pub fn new() -> Self {
        // TODO: Initialize with default prototype
        JSObject {
            _placeholder: 0,
        }
    }

    /// Returns the prototype
    pub fn prototype(&self) -> JSValue {
        // TODO: Return proto field
        JSValue::null()
    }

    /// Sets the prototype
    pub fn set_prototype(&mut self, _proto: JSValue) {
        // TODO: Set proto field
    }

    /// Returns the class ID
    pub fn class_id(&self) -> u8 {
        // TODO: Extract from header
        0
    }
}

impl Default for JSObject {
    fn default() -> Self {
        Self::new()
    }
}

/// Object class IDs
pub mod class_id {
    /// Generic object
    pub const OBJECT: u8 = 0;
    /// Array
    pub const ARRAY: u8 = 1;
    /// Function
    pub const FUNCTION: u8 = 2;
    /// String object
    pub const STRING: u8 = 3;
    /// Number object
    pub const NUMBER: u8 = 4;
    /// Boolean object
    pub const BOOLEAN: u8 = 5;
    /// Error
    pub const ERROR: u8 = 6;
    /// RegExp
    pub const REGEXP: u8 = 7;
    // TODO: Add more class IDs
}
