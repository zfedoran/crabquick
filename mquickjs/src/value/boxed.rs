//! Boxed Float64 values
//!
//! JavaScript numbers are 64-bit floats, but JSValue can only inline 31-bit integers.
//! Large integers and all floating-point values must be heap-allocated in a JSFloat64.

use core::fmt;

/// Boxed 64-bit floating point value
///
/// Layout in memory:
/// ```text
/// [MemBlockHeader][JSFloat64]
/// ```
///
/// JSFloat64 stores a single f64 value on the heap when it cannot be
/// represented as an inline integer in JSValue.
#[repr(C)]
pub struct JSFloat64 {
    /// The 64-bit floating point value
    value: f64,
}

impl JSFloat64 {
    /// Creates a new JSFloat64
    pub const fn new(value: f64) -> Self {
        JSFloat64 { value }
    }

    /// Returns the f64 value
    #[inline]
    pub const fn value(&self) -> f64 {
        self.value
    }

    /// Sets the f64 value
    #[inline]
    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }

    /// Returns the size of a JSFloat64 (excluding MemBlockHeader)
    #[inline]
    pub const fn alloc_size() -> usize {
        core::mem::size_of::<JSFloat64>()
    }

    /// Checks if a value can be represented as an inline integer
    ///
    /// Returns true if the value is a whole number in the range that fits
    /// in a 31-bit signed integer (JSValue can inline these).
    pub fn can_inline(value: f64) -> bool {
        // Check if it's a whole number
        if value.fract() != 0.0 {
            return false;
        }

        // Check if it fits in 31 bits
        const MIN: f64 = -0x4000_0000 as f64; // -2^30
        const MAX: f64 = 0x3FFF_FFFF as f64;  // 2^30 - 1

        value >= MIN && value <= MAX
    }

    /// Converts to i32 if possible
    ///
    /// Returns Some(i32) if the value is a whole number that fits in i32,
    /// None otherwise.
    pub fn to_i32(&self) -> Option<i32> {
        if Self::can_inline(self.value) {
            Some(self.value as i32)
        } else {
            None
        }
    }

    /// Checks if the value is NaN
    #[inline]
    pub fn is_nan(&self) -> bool {
        self.value.is_nan()
    }

    /// Checks if the value is infinite
    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.value.is_infinite()
    }

    /// Checks if the value is finite
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.value.is_finite()
    }
}

impl fmt::Debug for JSFloat64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSFloat64({})", self.value)
    }
}

impl fmt::Display for JSFloat64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PartialEq for JSFloat64 {
    fn eq(&self, other: &Self) -> bool {
        // Note: NaN != NaN in JavaScript
        if self.value.is_nan() && other.value.is_nan() {
            false
        } else {
            self.value == other.value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float64_creation() {
        let f = JSFloat64::new(3.14);
        assert_eq!(f.value(), 3.14);
    }

    #[test]
    fn test_float64_mutation() {
        let mut f = JSFloat64::new(1.0);
        assert_eq!(f.value(), 1.0);

        f.set_value(2.5);
        assert_eq!(f.value(), 2.5);
    }

    #[test]
    fn test_can_inline() {
        // Integers in range can be inlined
        assert!(JSFloat64::can_inline(0.0));
        assert!(JSFloat64::can_inline(42.0));
        assert!(JSFloat64::can_inline(-100.0));
        assert!(JSFloat64::can_inline(1000000.0));

        // Floats cannot be inlined
        assert!(!JSFloat64::can_inline(3.14));
        assert!(!JSFloat64::can_inline(0.5));
        assert!(!JSFloat64::can_inline(-2.7));

        // Large integers cannot be inlined (beyond 31-bit range)
        assert!(!JSFloat64::can_inline(2_000_000_000.0));
        assert!(!JSFloat64::can_inline(-2_000_000_000.0));

        // Special values cannot be inlined
        assert!(!JSFloat64::can_inline(f64::NAN));
        assert!(!JSFloat64::can_inline(f64::INFINITY));
        assert!(!JSFloat64::can_inline(f64::NEG_INFINITY));
    }

    #[test]
    fn test_to_i32() {
        let f1 = JSFloat64::new(42.0);
        assert_eq!(f1.to_i32(), Some(42));

        let f2 = JSFloat64::new(3.14);
        assert_eq!(f2.to_i32(), None);

        let f3 = JSFloat64::new(2_000_000_000.0);
        assert_eq!(f3.to_i32(), None);
    }

    #[test]
    fn test_special_values() {
        let nan = JSFloat64::new(f64::NAN);
        assert!(nan.is_nan());
        assert!(!nan.is_finite());

        let inf = JSFloat64::new(f64::INFINITY);
        assert!(inf.is_infinite());
        assert!(!inf.is_finite());

        let normal = JSFloat64::new(3.14);
        assert!(!normal.is_nan());
        assert!(!normal.is_infinite());
        assert!(normal.is_finite());
    }

    #[test]
    fn test_nan_equality() {
        let nan1 = JSFloat64::new(f64::NAN);
        let nan2 = JSFloat64::new(f64::NAN);
        assert_ne!(nan1, nan2); // NaN != NaN
    }

    #[test]
    fn test_alloc_size() {
        assert_eq!(JSFloat64::alloc_size(), 8);
    }
}
