//! JavaScript class system
//!
//! Defines the class system for JavaScript objects. Each object has a class ID
//! that determines its behavior and internal data layout.

/// JavaScript class IDs
///
/// Each object has a class ID that determines its type and behavior.
/// The class ID is stored in the object header (8 bits).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum JSClassID {
    /// Generic object (Object)
    Object = 0,
    /// Array object (Array)
    Array = 1,
    /// Function object
    Function = 2,
    /// String object (boxed string wrapper)
    String = 3,
    /// Number object (boxed number wrapper)
    Number = 4,
    /// Boolean object (boxed boolean wrapper)
    Boolean = 5,
    /// Error object
    Error = 6,
    /// RegExp object
    RegExp = 7,
    /// Date object
    Date = 8,
    /// Math object (singleton)
    Math = 9,
    /// JSON object (singleton)
    JSON = 10,
    /// Arguments object
    Arguments = 11,
    /// ArrayBuffer
    ArrayBuffer = 12,
    /// DataView
    DataView = 13,
    /// Int8Array
    Int8Array = 14,
    /// Uint8Array
    Uint8Array = 15,
    /// Uint8ClampedArray
    Uint8ClampedArray = 16,
    /// Int16Array
    Int16Array = 17,
    /// Uint16Array
    Uint16Array = 18,
    /// Int32Array
    Int32Array = 19,
    /// Uint32Array
    Uint32Array = 20,
    /// Float32Array
    Float32Array = 21,
    /// Float64Array
    Float64Array = 22,
}

impl JSClassID {
    /// Creates a class ID from a u8 value
    ///
    /// Returns None if the value doesn't correspond to a valid class ID.
    #[inline]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(JSClassID::Object),
            1 => Some(JSClassID::Array),
            2 => Some(JSClassID::Function),
            3 => Some(JSClassID::String),
            4 => Some(JSClassID::Number),
            5 => Some(JSClassID::Boolean),
            6 => Some(JSClassID::Error),
            7 => Some(JSClassID::RegExp),
            8 => Some(JSClassID::Date),
            9 => Some(JSClassID::Math),
            10 => Some(JSClassID::JSON),
            11 => Some(JSClassID::Arguments),
            12 => Some(JSClassID::ArrayBuffer),
            13 => Some(JSClassID::DataView),
            14 => Some(JSClassID::Int8Array),
            15 => Some(JSClassID::Uint8Array),
            16 => Some(JSClassID::Uint8ClampedArray),
            17 => Some(JSClassID::Int16Array),
            18 => Some(JSClassID::Uint16Array),
            19 => Some(JSClassID::Int32Array),
            20 => Some(JSClassID::Uint32Array),
            21 => Some(JSClassID::Float32Array),
            22 => Some(JSClassID::Float64Array),
            _ => None,
        }
    }

    /// Returns true if this is a typed array class
    #[inline]
    pub fn is_typed_array(&self) -> bool {
        matches!(
            self,
            JSClassID::Int8Array
                | JSClassID::Uint8Array
                | JSClassID::Uint8ClampedArray
                | JSClassID::Int16Array
                | JSClassID::Uint16Array
                | JSClassID::Int32Array
                | JSClassID::Uint32Array
                | JSClassID::Float32Array
                | JSClassID::Float64Array
        )
    }

    /// Returns true if this is an error class
    #[inline]
    pub fn is_error(&self) -> bool {
        *self == JSClassID::Error
    }

    /// Returns true if this is a function class
    #[inline]
    pub fn is_function(&self) -> bool {
        *self == JSClassID::Function
    }

    /// Returns true if this is an array class
    #[inline]
    pub fn is_array(&self) -> bool {
        *self == JSClassID::Array
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_id_from_u8() {
        assert_eq!(JSClassID::from_u8(0), Some(JSClassID::Object));
        assert_eq!(JSClassID::from_u8(1), Some(JSClassID::Array));
        assert_eq!(JSClassID::from_u8(2), Some(JSClassID::Function));
        assert_eq!(JSClassID::from_u8(255), None);
    }

    #[test]
    fn test_class_id_as_u8() {
        assert_eq!(JSClassID::Object as u8, 0);
        assert_eq!(JSClassID::Array as u8, 1);
        assert_eq!(JSClassID::Function as u8, 2);
    }

    #[test]
    fn test_is_typed_array() {
        assert!(!JSClassID::Object.is_typed_array());
        assert!(!JSClassID::Array.is_typed_array());
        assert!(JSClassID::Int8Array.is_typed_array());
        assert!(JSClassID::Uint8Array.is_typed_array());
        assert!(JSClassID::Float64Array.is_typed_array());
    }

    #[test]
    fn test_is_error() {
        assert!(!JSClassID::Object.is_error());
        assert!(JSClassID::Error.is_error());
    }

    #[test]
    fn test_is_function() {
        assert!(!JSClassID::Object.is_function());
        assert!(JSClassID::Function.is_function());
    }

    #[test]
    fn test_is_array() {
        assert!(!JSClassID::Object.is_array());
        assert!(JSClassID::Array.is_array());
    }
}
