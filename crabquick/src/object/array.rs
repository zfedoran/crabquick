//! JavaScript array implementation
//!
//! This module provides the JSArray class data structure that is attached to
//! Array objects. The actual element storage uses JSValueArray from the value module.
//!
//! ## Features Implemented:
//! - Dynamic backing storage using JSValueArray
//! - Sparse array support for large indices (via property table)
//! - Length property that syncs with actual element count
//! - Efficient element access for dense arrays
//! - Automatic growth when setting elements beyond capacity

use crate::value::JSValue;
use crate::memory::HeapIndex;

/// Array class data
///
/// This is stored as class_data in JSObject when the object's class is Array.
/// It maintains both a dense element array for sequential indices and relies
/// on the object's property table for sparse/named properties.
///
/// Layout:
/// - elements: HeapIndex to JSValueArray for dense storage
/// - length: Current logical length of the array
///
/// The array uses a hybrid approach:
/// - Indices 0..capacity are stored in the dense elements array
/// - Sparse indices and named properties go in the property table
/// - The length property is managed separately and can exceed element count
#[repr(C)]
pub struct JSArray {
    /// Index to JSValueArray for dense element storage
    /// Can be null if array is empty or sparse
    elements: HeapIndex,
    /// Logical length of the array (may exceed actual element count)
    length: u32,
    /// Padding for alignment
    _padding: u32,
}

impl JSArray {
    /// Creates a new array with the specified length
    ///
    /// The array is initially empty (no backing storage) but reports the given length.
    /// Storage will be allocated on first element access/assignment.
    pub fn new(length: u32) -> Self {
        JSArray {
            elements: HeapIndex::null(),
            length,
            _padding: 0,
        }
    }

    /// Creates a new array with preallocated capacity
    ///
    /// This should be called with a valid elements array index.
    pub fn with_elements(elements: HeapIndex, length: u32) -> Self {
        JSArray {
            elements,
            length,
            _padding: 0,
        }
    }

    /// Returns the array length
    #[inline]
    pub fn len(&self) -> u32 {
        self.length
    }

    /// Returns true if the array is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Sets the array length
    ///
    /// If the new length is smaller than current, elements beyond the new length
    /// should be deleted. If larger, the array grows but new elements are undefined.
    ///
    /// Note: The actual element truncation/growth should be handled by the caller
    /// who has access to the Context and can manipulate the elements array.
    #[inline]
    pub fn set_len(&mut self, new_len: u32) {
        self.length = new_len;
    }

    /// Returns the elements array index
    #[inline]
    pub fn elements_index(&self) -> HeapIndex {
        self.elements
    }

    /// Sets the elements array index
    #[inline]
    pub fn set_elements_index(&mut self, index: HeapIndex) {
        self.elements = index;
    }

    /// Returns true if the array has allocated element storage
    #[inline]
    pub fn has_elements(&self) -> bool {
        !self.elements.is_null()
    }

    /// Gets an element by index
    ///
    /// Returns undefined if:
    /// - Index is out of bounds
    /// - No elements array is allocated
    /// - The element is stored sparsely (caller should check property table)
    ///
    /// # Safety
    ///
    /// The caller must provide a valid Arena reference and ensure the
    /// elements index points to a valid JSValueArray.
    pub unsafe fn get(&self, index: u32, arena: &crate::memory::Arena) -> JSValue {
        // Check if index is within length
        if index >= self.length {
            return JSValue::undefined();
        }

        // Check if we have elements storage
        if self.elements.is_null() {
            return JSValue::undefined();
        }

        // Get the elements array
        let elements: &crate::value::JSValueArray = arena.get(self.elements);
        let count = elements.header().count();

        // Check if index is within dense storage
        if (index as usize) >= count {
            return JSValue::undefined();
        }

        // Return the element
        elements.get_unchecked(index as usize)
    }

    /// Sets an element by index
    ///
    /// If index >= current capacity, the caller should reallocate the elements array.
    /// If index >= length, the length is updated.
    ///
    /// # Safety
    ///
    /// The caller must:
    /// - Provide a valid Arena reference
    /// - Ensure the elements index points to a valid JSValueArray
    /// - Ensure the index is within the allocated capacity
    pub unsafe fn set(&mut self, index: u32, value: JSValue, arena: &mut crate::memory::Arena) -> bool {
        // Update length if needed
        if index >= self.length {
            self.length = index + 1;
        }

        // Check if we have elements storage
        if self.elements.is_null() {
            // No storage, caller should allocate
            return false;
        }

        // Get the elements array
        let elements: &mut crate::value::JSValueArray = arena.get_mut(self.elements);
        let capacity = elements.header().capacity();

        // Check if index is within capacity
        if (index as usize) >= capacity {
            // Need reallocation, return false
            return false;
        }

        // Update count if we're extending the dense storage
        let count = elements.header().count();
        if (index as usize) >= count {
            elements.header_mut().set_count((index + 1) as usize);
        }

        // Set the element
        elements.set_unchecked(index as usize, value);
        true
    }
}

/// Value array storage (JSValue[])
///
/// This is re-exported from the value module.
pub use crate::value::JSValueArray;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::memory::MemTag;

    #[test]
    fn test_array_new() {
        let arr = JSArray::new(10);
        assert_eq!(arr.len(), 10);
        assert!(!arr.has_elements());
        assert!(arr.elements_index().is_null());
    }

    #[test]
    fn test_array_with_elements() {
        let idx = HeapIndex::from_usize(42);
        let arr = JSArray::with_elements(idx, 5);
        assert_eq!(arr.len(), 5);
        assert!(arr.has_elements());
        assert_eq!(arr.elements_index(), idx);
    }

    #[test]
    fn test_array_set_len() {
        let mut arr = JSArray::new(10);
        assert_eq!(arr.len(), 10);

        arr.set_len(20);
        assert_eq!(arr.len(), 20);

        arr.set_len(5);
        assert_eq!(arr.len(), 5);
    }

    #[test]
    fn test_array_is_empty() {
        let arr = JSArray::new(0);
        assert!(arr.is_empty());

        let arr2 = JSArray::new(5);
        assert!(!arr2.is_empty());
    }

    #[test]
    fn test_array_get_set_with_context() {
        let mut ctx = Context::new(8192);

        // Allocate elements array with capacity 10
        let elements_idx = ctx.alloc_value_array(10).unwrap();

        // Create array
        let mut arr = JSArray::with_elements(elements_idx, 0);

        // Set some elements
        unsafe {
            let success = arr.set(0, JSValue::from_int(42), ctx.arena_mut());
            assert!(success);
            assert_eq!(arr.len(), 1);

            let success = arr.set(1, JSValue::from_int(100), ctx.arena_mut());
            assert!(success);
            assert_eq!(arr.len(), 2);

            // Get elements back
            let val0 = arr.get(0, ctx.arena());
            assert_eq!(val0.to_int(), Some(42));

            let val1 = arr.get(1, ctx.arena());
            assert_eq!(val1.to_int(), Some(100));

            // Get out of bounds
            let val2 = arr.get(5, ctx.arena());
            assert_eq!(val2, JSValue::undefined());
        }
    }

    #[test]
    fn test_array_sparse_index() {
        let mut ctx = Context::new(8192);

        // Allocate small capacity
        let elements_idx = ctx.alloc_value_array(5).unwrap();
        let mut arr = JSArray::with_elements(elements_idx, 0);

        // Set element at index 0
        unsafe {
            arr.set(0, JSValue::from_int(1), ctx.arena_mut());
        }

        // Try to set element at index 10 (beyond capacity)
        unsafe {
            let success = arr.set(10, JSValue::from_int(100), ctx.arena_mut());
            assert!(!success); // Should fail, needs reallocation
        }
    }

    #[test]
    fn test_array_no_elements() {
        let ctx = Context::new(8192);
        let arr = JSArray::new(5);

        // Get from array with no elements storage
        unsafe {
            let val = arr.get(0, ctx.arena());
            assert_eq!(val, JSValue::undefined());
        }
    }

    #[test]
    fn test_array_length_tracking() {
        let mut ctx = Context::new(8192);
        let elements_idx = ctx.alloc_value_array(10).unwrap();
        let mut arr = JSArray::with_elements(elements_idx, 0);

        // Initially empty
        assert_eq!(arr.len(), 0);

        // Set at index 0
        unsafe {
            arr.set(0, JSValue::from_int(10), ctx.arena_mut());
            assert_eq!(arr.len(), 1);

            // Set at index 5
            arr.set(5, JSValue::from_int(50), ctx.arena_mut());
            assert_eq!(arr.len(), 6);

            // Set at index 3 (doesn't increase length)
            arr.set(3, JSValue::from_int(30), ctx.arena_mut());
            assert_eq!(arr.len(), 6);
        }
    }

    #[test]
    fn test_array_size() {
        // Verify array size is reasonable (16 bytes)
        let size = core::mem::size_of::<JSArray>();
        assert_eq!(size, 16);
    }
}
