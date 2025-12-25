//! Constant pool management
//!
//! The constant pool stores literal values referenced by bytecode instructions.
//! Constants are deduplicated to save space - identical values share the same index.

use crate::value::JSValue;
use alloc::vec::Vec;

/// Constant pool for bytecode functions
///
/// Stores literal values (numbers, strings, nested functions, etc.) that are
/// referenced by bytecode instructions via indices. The pool automatically
/// deduplicates constants to minimize memory usage.
pub struct ConstantPool {
    /// The constant values
    constants: Vec<JSValue>,
}

impl ConstantPool {
    /// Creates a new empty constant pool
    pub fn new() -> Self {
        ConstantPool {
            constants: Vec::new(),
        }
    }

    /// Creates a new constant pool with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        ConstantPool {
            constants: Vec::with_capacity(capacity),
        }
    }

    /// Adds a constant to the pool
    ///
    /// If the constant already exists, returns the existing index.
    /// Otherwise, adds the constant and returns the new index.
    ///
    /// Returns None if the pool is full (> 65535 constants).
    pub fn add(&mut self, value: JSValue) -> Option<u16> {
        // Check if constant already exists
        if let Some(index) = self.find(value) {
            return Some(index);
        }

        // Check if pool is full
        if self.constants.len() >= 65536 {
            return None;
        }

        // Add new constant
        let index = self.constants.len() as u16;
        self.constants.push(value);
        Some(index)
    }

    /// Finds a constant in the pool
    ///
    /// Returns the index if found, None otherwise.
    fn find(&self, value: JSValue) -> Option<u16> {
        // For simple equality comparison
        // Note: This uses bitwise equality which works for our tagged values
        let value_bits = value.as_raw();
        self.constants
            .iter()
            .position(|v: &JSValue| v.as_raw() == value_bits)
            .map(|pos| pos as u16)
    }

    /// Gets a constant by index
    pub fn get(&self, index: u16) -> Option<JSValue> {
        self.constants.get(index as usize).copied()
    }

    /// Returns the number of constants
    pub fn len(&self) -> usize {
        self.constants.len()
    }

    /// Returns true if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.constants.is_empty()
    }

    /// Returns a reference to the constants
    pub fn as_slice(&self) -> &[JSValue] {
        &self.constants
    }

    /// Clears the constant pool
    pub fn clear(&mut self) {
        self.constants.clear();
    }

    /// Reserves capacity for at least additional more constants
    pub fn reserve(&mut self, additional: usize) {
        self.constants.reserve(additional);
    }
}

impl Default for ConstantPool {
    fn default() -> Self {
        Self::new()
    }
}

// Implement FromIterator for convenient construction
impl core::iter::FromIterator<JSValue> for ConstantPool {
    fn from_iter<T: IntoIterator<Item = JSValue>>(iter: T) -> Self {
        ConstantPool {
            constants: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let pool = ConstantPool::new();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_add_and_get() {
        let mut pool = ConstantPool::new();

        let val1 = JSValue::from_int(42);
        let val2 = JSValue::from_int(100);

        let idx1 = pool.add(val1).unwrap();
        assert_eq!(idx1, 0);
        assert_eq!(pool.len(), 1);

        let idx2 = pool.add(val2).unwrap();
        assert_eq!(idx2, 1);
        assert_eq!(pool.len(), 2);

        assert_eq!(pool.get(idx1), Some(val1));
        assert_eq!(pool.get(idx2), Some(val2));
    }

    #[test]
    fn test_deduplication() {
        let mut pool = ConstantPool::new();

        let val = JSValue::from_int(42);

        let idx1 = pool.add(val).unwrap();
        let idx2 = pool.add(val).unwrap();
        let idx3 = pool.add(val).unwrap();

        // All three should return the same index
        assert_eq!(idx1, idx2);
        assert_eq!(idx2, idx3);

        // Pool should only contain one constant
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_different_values() {
        let mut pool = ConstantPool::new();

        let val1 = JSValue::from_int(1);
        let val2 = JSValue::from_int(2);
        let val3 = JSValue::from_int(3);

        let idx1 = pool.add(val1).unwrap();
        let idx2 = pool.add(val2).unwrap();
        let idx3 = pool.add(val3).unwrap();

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);
        assert_eq!(pool.len(), 3);
    }

    #[test]
    fn test_special_values() {
        let mut pool = ConstantPool::new();

        let undefined = JSValue::undefined();
        let null = JSValue::null();
        let true_val = JSValue::bool(true);
        let false_val = JSValue::bool(false);

        let idx1 = pool.add(undefined).unwrap();
        let idx2 = pool.add(null).unwrap();
        let idx3 = pool.add(true_val).unwrap();
        let idx4 = pool.add(false_val).unwrap();

        // Each should get a unique index
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);
        assert_eq!(idx4, 3);

        // Verify retrieval
        assert_eq!(pool.get(idx1), Some(undefined));
        assert_eq!(pool.get(idx2), Some(null));
        assert_eq!(pool.get(idx3), Some(true_val));
        assert_eq!(pool.get(idx4), Some(false_val));
    }

    #[test]
    fn test_get_invalid_index() {
        let pool = ConstantPool::new();
        assert_eq!(pool.get(0), None);
        assert_eq!(pool.get(100), None);
    }

    #[test]
    fn test_clear() {
        let mut pool = ConstantPool::new();

        pool.add(JSValue::from_int(1)).unwrap();
        pool.add(JSValue::from_int(2)).unwrap();
        assert_eq!(pool.len(), 2);

        pool.clear();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_as_slice() {
        let mut pool = ConstantPool::new();

        let val1 = JSValue::from_int(10);
        let val2 = JSValue::from_int(20);
        let val3 = JSValue::from_int(30);

        pool.add(val1).unwrap();
        pool.add(val2).unwrap();
        pool.add(val3).unwrap();

        let slice = pool.as_slice();
        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], val1);
        assert_eq!(slice[1], val2);
        assert_eq!(slice[2], val3);
    }

    #[test]
    fn test_with_capacity() {
        let pool = ConstantPool::with_capacity(10);
        assert_eq!(pool.len(), 0);
        assert!(pool.constants.capacity() >= 10);
    }

    #[test]
    fn test_from_iter() {
        let values = vec![
            JSValue::from_int(1),
            JSValue::from_int(2),
            JSValue::from_int(3),
        ];

        let pool: ConstantPool = values.into_iter().collect();
        assert_eq!(pool.len(), 3);
    }

    #[test]
    fn test_reserve() {
        let mut pool = ConstantPool::new();
        pool.reserve(100);
        assert!(pool.constants.capacity() >= 100);
    }

    #[test]
    fn test_mixed_deduplication() {
        let mut pool = ConstantPool::new();

        let val1 = JSValue::from_int(42);
        let val2 = JSValue::from_int(100);

        // Add val1
        let idx1 = pool.add(val1).unwrap();
        assert_eq!(idx1, 0);

        // Add val2
        let idx2 = pool.add(val2).unwrap();
        assert_eq!(idx2, 1);

        // Add val1 again (should deduplicate)
        let idx3 = pool.add(val1).unwrap();
        assert_eq!(idx3, 0);

        // Add val2 again (should deduplicate)
        let idx4 = pool.add(val2).unwrap();
        assert_eq!(idx4, 1);

        // Pool should only have 2 constants
        assert_eq!(pool.len(), 2);
    }
}
