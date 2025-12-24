//! JavaScript value representation
//!
//! JSValue uses tagged encoding to represent all JavaScript types in a single
//! machine word (32 or 64 bits). This includes integers, pointers to heap objects,
//! and special values like null, undefined, and booleans.

/// JavaScript value with tagged encoding
///
/// ## Encoding Scheme (32-bit)
///
/// - Integers: 31-bit signed integer with LSB = 0
/// - Pointers: Heap index with tag bits 0b001
/// - Special values: null, undefined, true, false with tag 0b011
///
/// ## Encoding Scheme (64-bit)
///
/// - Integers: 31-bit signed integer with LSB = 0
/// - Short floats: 48-bit float with special encoding
/// - Pointers: Heap index with tag bits
/// - Special values: null, undefined, true, false
///
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct JSValue(usize);

impl JSValue {
    // Tag constants
    const TAG_MASK: usize = 0x7;
    const TAG_INT: usize = 0;      // 0b000 - integers (LSB is always 0)
    const TAG_PTR: usize = 1;      // 0b001 - heap pointers
    const TAG_SPECIAL: usize = 3;  // 0b011 - special values

    // Special value encodings
    const VAL_NULL: usize = (0 << 3) | Self::TAG_SPECIAL;
    const VAL_UNDEFINED: usize = (1 << 3) | Self::TAG_SPECIAL;
    const VAL_FALSE: usize = (2 << 3) | Self::TAG_SPECIAL;
    const VAL_TRUE: usize = (3 << 3) | Self::TAG_SPECIAL;
    const VAL_EXCEPTION: usize = (4 << 3) | Self::TAG_SPECIAL;

    /// Creates a JSValue from a 32-bit signed integer
    ///
    /// The integer must fit in 31 bits (range: -2^30 to 2^30-1).
    /// Values outside this range must be boxed as heap-allocated float64.
    #[inline]
    pub const fn from_int(i: i32) -> Self {
        JSValue((i as usize) << 1)
    }

    /// Attempts to extract an integer from this value
    ///
    /// Returns None if the value is not an integer.
    #[inline]
    pub const fn to_int(self) -> Option<i32> {
        if (self.0 & 1) == 0 {
            Some((self.0 as isize >> 1) as i32)
        } else {
            None
        }
    }

    /// Returns true if this value is an integer
    #[inline]
    pub const fn is_int(self) -> bool {
        (self.0 & 1) == 0
    }

    /// Creates the null value
    #[inline]
    pub const fn null() -> Self {
        JSValue(Self::VAL_NULL)
    }

    /// Creates the undefined value
    #[inline]
    pub const fn undefined() -> Self {
        JSValue(Self::VAL_UNDEFINED)
    }

    /// Creates a boolean value
    #[inline]
    pub const fn bool(b: bool) -> Self {
        if b {
            JSValue(Self::VAL_TRUE)
        } else {
            JSValue(Self::VAL_FALSE)
        }
    }

    /// Creates the exception marker value
    #[inline]
    pub const fn exception() -> Self {
        JSValue(Self::VAL_EXCEPTION)
    }

    /// Returns true if this value is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == Self::VAL_NULL
    }

    /// Returns true if this value is undefined
    #[inline]
    pub const fn is_undefined(self) -> bool {
        self.0 == Self::VAL_UNDEFINED
    }

    /// Returns true if this value is a boolean
    #[inline]
    pub const fn is_bool(self) -> bool {
        self.0 == Self::VAL_FALSE || self.0 == Self::VAL_TRUE
    }

    /// Returns true if this value is the exception marker
    #[inline]
    pub const fn is_exception(self) -> bool {
        self.0 == Self::VAL_EXCEPTION
    }

    /// Extracts a boolean value
    ///
    /// Returns None if the value is not a boolean.
    #[inline]
    pub const fn to_bool(self) -> Option<bool> {
        if self.0 == Self::VAL_TRUE {
            Some(true)
        } else if self.0 == Self::VAL_FALSE {
            Some(false)
        } else {
            None
        }
    }

    /// Creates a JSValue from a heap pointer
    #[inline]
    pub const fn from_ptr(index: crate::memory::HeapIndex) -> Self {
        JSValue((index.0 as usize) << 3 | Self::TAG_PTR)
    }

    /// Attempts to extract a heap pointer from this value
    ///
    /// Returns None if the value is not a pointer.
    #[inline]
    pub const fn to_ptr(self) -> Option<crate::memory::HeapIndex> {
        if (self.0 & Self::TAG_MASK) == Self::TAG_PTR {
            Some(crate::memory::HeapIndex((self.0 >> 3) as u32))
        } else {
            None
        }
    }

    /// Returns true if this value is a heap pointer
    #[inline]
    pub const fn is_ptr(self) -> bool {
        (self.0 & Self::TAG_MASK) == Self::TAG_PTR
    }

    /// Returns true if this value is an object
    ///
    /// Note: This checks if it's a pointer. More specific type checking
    /// requires inspecting the object header.
    #[inline]
    pub const fn is_object(self) -> bool {
        self.is_ptr()
    }
}

// Implement common traits
impl PartialEq for JSValue {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Implement JavaScript equality semantics
        self.0 == other.0
    }
}

impl Eq for JSValue {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_encoding() {
        let val = JSValue::from_int(42);
        assert_eq!(val.to_int(), Some(42));
        assert!(val.is_int());
    }

    #[test]
    fn test_int_negative() {
        let val = JSValue::from_int(-100);
        assert_eq!(val.to_int(), Some(-100));
    }

    #[test]
    fn test_special_values() {
        assert!(JSValue::null().is_null());
        assert!(JSValue::undefined().is_undefined());
        assert!(JSValue::bool(true).is_bool());
        assert!(JSValue::bool(false).is_bool());
        assert_eq!(JSValue::bool(true).to_bool(), Some(true));
        assert_eq!(JSValue::bool(false).to_bool(), Some(false));
    }

    #[test]
    fn test_exception() {
        let ex = JSValue::exception();
        assert!(ex.is_exception());
    }

    #[test]
    fn test_ptr_encoding() {
        use crate::memory::HeapIndex;

        let idx = HeapIndex::from_usize(0);
        let val = JSValue::from_ptr(idx);

        assert!(val.is_ptr());
        assert!(val.is_object());
        assert_eq!(val.to_ptr(), Some(idx));
        assert!(!val.is_int());
        assert!(!val.is_null());
    }

    #[test]
    fn test_ptr_various_indices() {
        use crate::memory::HeapIndex;

        for offset in [0, 8, 16, 64, 128, 256, 1024, 4096] {
            let idx = HeapIndex::from_usize(offset);
            let val = JSValue::from_ptr(idx);

            assert!(val.is_ptr());
            assert_eq!(val.to_ptr(), Some(idx));
            assert_eq!(val.to_ptr().unwrap().as_usize(), offset);
        }
    }

    #[test]
    fn test_value_type_distinction() {
        use crate::memory::HeapIndex;

        // Integer
        let int_val = JSValue::from_int(42);
        assert!(int_val.is_int());
        assert!(!int_val.is_ptr());
        assert!(!int_val.is_null());
        assert!(!int_val.is_bool());

        // Pointer
        let ptr_val = JSValue::from_ptr(HeapIndex::from_usize(64));
        assert!(!ptr_val.is_int());
        assert!(ptr_val.is_ptr());
        assert!(!ptr_val.is_null());
        assert!(!ptr_val.is_bool());

        // Null
        let null_val = JSValue::null();
        assert!(!null_val.is_int());
        assert!(!null_val.is_ptr());
        assert!(null_val.is_null());
        assert!(!null_val.is_bool());

        // Boolean
        let bool_val = JSValue::bool(true);
        assert!(!bool_val.is_int());
        assert!(!bool_val.is_ptr());
        assert!(!bool_val.is_null());
        assert!(bool_val.is_bool());
    }
}
