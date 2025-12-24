//! JavaScript array types
//!
//! This module provides two types of arrays:
//! - JSValueArray: Array of JSValue (for JavaScript arrays and property tables)
//! - JSByteArray: Array of bytes (for typed arrays, strings, bytecode)

use crate::value::JSValue;

/// Array of JSValue
///
/// Layout in memory:
/// ```text
/// [MemBlockHeader][JSValueArrayHeader][JSValue, JSValue, ...]
/// ```
#[repr(C)]
pub struct JSValueArrayHeader {
    /// Current number of elements
    count: u32,
    /// Allocated capacity
    capacity: u32,
}

impl JSValueArrayHeader {
    /// Creates a new value array header
    pub fn new(capacity: usize) -> Self {
        JSValueArrayHeader {
            count: 0,
            capacity: capacity as u32,
        }
    }

    /// Returns the current count
    #[inline]
    pub fn count(&self) -> usize {
        self.count as usize
    }

    /// Returns the capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity as usize
    }

    /// Sets the count
    #[inline]
    pub fn set_count(&mut self, count: usize) {
        self.count = count as u32;
    }

    /// Returns the total allocation size (header + data)
    #[inline]
    pub fn alloc_size(&self) -> usize {
        size_of::<crate::memory::MemBlockHeader>()
            + size_of::<JSValueArrayHeader>()
            + self.capacity() * size_of::<JSValue>()
    }
}

/// JavaScript value array
pub struct JSValueArray {
    header: JSValueArrayHeader,
    // JSValue elements follow here (flexible array member)
}

impl JSValueArray {
    /// Returns the header size
    #[inline]
    pub const fn header_size() -> usize {
        size_of::<JSValueArrayHeader>()
    }

    /// Returns the total size needed for an array allocation (excluding MemBlockHeader)
    #[inline]
    pub const fn alloc_size(capacity: usize) -> usize {
        size_of::<JSValueArrayHeader>() + capacity * size_of::<JSValue>()
    }

    /// Returns the header
    #[inline]
    pub fn header(&self) -> &JSValueArrayHeader {
        &self.header
    }

    /// Returns the mutable header
    #[inline]
    pub fn header_mut(&mut self) -> &mut JSValueArrayHeader {
        &mut self.header
    }

    /// Returns the element slice
    ///
    /// # Safety
    ///
    /// The caller must ensure this array was properly allocated.
    #[inline]
    pub unsafe fn as_slice(&self) -> &[JSValue] {
        let ptr = (self as *const Self as *const u8).add(size_of::<JSValueArrayHeader>());
        core::slice::from_raw_parts(ptr as *const JSValue, self.header.count())
    }

    /// Returns the mutable element slice
    ///
    /// # Safety
    ///
    /// The caller must ensure this array was properly allocated.
    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [JSValue] {
        let ptr = (self as *mut Self as *mut u8).add(size_of::<JSValueArrayHeader>());
        core::slice::from_raw_parts_mut(ptr as *mut JSValue, self.header.count())
    }

    /// Returns the full capacity slice (including unused elements)
    ///
    /// # Safety
    ///
    /// The caller must ensure this array was properly allocated.
    #[inline]
    pub unsafe fn as_full_slice(&self) -> &[JSValue] {
        let ptr = (self as *const Self as *const u8).add(size_of::<JSValueArrayHeader>());
        core::slice::from_raw_parts(ptr as *const JSValue, self.header.capacity())
    }

    /// Returns the mutable full capacity slice
    ///
    /// # Safety
    ///
    /// The caller must ensure this array was properly allocated.
    #[inline]
    pub unsafe fn as_full_mut_slice(&mut self) -> &mut [JSValue] {
        let ptr = (self as *mut Self as *mut u8).add(size_of::<JSValueArrayHeader>());
        core::slice::from_raw_parts_mut(ptr as *mut JSValue, self.header.capacity())
    }

    /// Gets an element at the specified index
    ///
    /// # Safety
    ///
    /// The caller must ensure the index is within bounds.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> JSValue {
        let slice = self.as_slice();
        *slice.get_unchecked(index)
    }

    /// Sets an element at the specified index
    ///
    /// # Safety
    ///
    /// The caller must ensure the index is within bounds.
    #[inline]
    pub unsafe fn set_unchecked(&mut self, index: usize, value: JSValue) {
        let slice = self.as_mut_slice();
        *slice.get_unchecked_mut(index) = value;
    }

    /// Pushes a value to the array (if capacity allows)
    ///
    /// Returns true if successful, false if at capacity.
    ///
    /// # Safety
    ///
    /// The caller must ensure the array was properly allocated.
    pub unsafe fn push(&mut self, value: JSValue) -> bool {
        let count = self.header.count();
        let capacity = self.header.capacity();

        if count >= capacity {
            return false;
        }

        let slice = self.as_full_mut_slice();
        slice[count] = value;
        self.header.set_count(count + 1);
        true
    }

    /// Pops a value from the array
    ///
    /// Returns None if the array is empty.
    ///
    /// # Safety
    ///
    /// The caller must ensure the array was properly allocated.
    pub unsafe fn pop(&mut self) -> Option<JSValue> {
        let count = self.header.count();
        if count == 0 {
            return None;
        }

        let slice = self.as_slice();
        let value = slice[count - 1];
        self.header.set_count(count - 1);
        Some(value)
    }

    /// Removes and returns the first element from the array (shift operation)
    ///
    /// Returns None if the array is empty.
    ///
    /// # Safety
    ///
    /// The caller must ensure the array was properly allocated.
    pub unsafe fn shift(&mut self) -> Option<JSValue> {
        let count = self.header.count();
        if count == 0 {
            return None;
        }

        let slice = self.as_mut_slice();
        let value = slice[0];

        // Shift all elements down by one
        for i in 1..count {
            slice[i - 1] = slice[i];
        }

        self.header.set_count(count - 1);
        Some(value)
    }

    /// Adds an element to the beginning of the array (unshift operation)
    ///
    /// Returns false if the array is full.
    ///
    /// # Safety
    ///
    /// The caller must ensure the array was properly allocated.
    pub unsafe fn unshift(&mut self, value: JSValue) -> bool {
        let count = self.header.count();
        let capacity = self.header.capacity();

        if count >= capacity {
            return false;
        }

        let slice = self.as_mut_slice();

        // Shift all elements up by one
        for i in (0..count).rev() {
            slice[i + 1] = slice[i];
        }

        // Insert new element at the beginning
        let full_slice = self.as_full_mut_slice();
        full_slice[0] = value;

        self.header.set_count(count + 1);
        true
    }
}

/// Array of bytes
///
/// Layout in memory:
/// ```text
/// [MemBlockHeader][JSByteArrayHeader][u8, u8, u8, ...]
/// ```
#[repr(C)]
pub struct JSByteArrayHeader {
    /// Current number of bytes
    count: u32,
    /// Allocated capacity
    capacity: u32,
}

impl JSByteArrayHeader {
    /// Creates a new byte array header
    pub fn new(capacity: usize) -> Self {
        JSByteArrayHeader {
            count: 0,
            capacity: capacity as u32,
        }
    }

    /// Returns the current count
    #[inline]
    pub fn count(&self) -> usize {
        self.count as usize
    }

    /// Returns the capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity as usize
    }

    /// Sets the count
    #[inline]
    pub fn set_count(&mut self, count: usize) {
        self.count = count as u32;
    }

    /// Returns the total allocation size (header + data)
    #[inline]
    pub fn alloc_size(&self) -> usize {
        size_of::<crate::memory::MemBlockHeader>()
            + size_of::<JSByteArrayHeader>()
            + self.capacity()
    }
}

/// JavaScript byte array
pub struct JSByteArray {
    header: JSByteArrayHeader,
    // u8 elements follow here (flexible array member)
}

impl JSByteArray {
    /// Returns the header size
    #[inline]
    pub const fn header_size() -> usize {
        size_of::<JSByteArrayHeader>()
    }

    /// Returns the total size needed for a byte array allocation (excluding MemBlockHeader)
    #[inline]
    pub const fn alloc_size(capacity: usize) -> usize {
        size_of::<JSByteArrayHeader>() + capacity
    }

    /// Returns the header
    #[inline]
    pub fn header(&self) -> &JSByteArrayHeader {
        &self.header
    }

    /// Returns the mutable header
    #[inline]
    pub fn header_mut(&mut self) -> &mut JSByteArrayHeader {
        &mut self.header
    }

    /// Returns the byte slice
    ///
    /// # Safety
    ///
    /// The caller must ensure this array was properly allocated.
    #[inline]
    pub unsafe fn as_slice(&self) -> &[u8] {
        let ptr = (self as *const Self as *const u8).add(size_of::<JSByteArrayHeader>());
        core::slice::from_raw_parts(ptr, self.header.count())
    }

    /// Returns the mutable byte slice
    ///
    /// # Safety
    ///
    /// The caller must ensure this array was properly allocated.
    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        let ptr = (self as *mut Self as *mut u8).add(size_of::<JSByteArrayHeader>());
        core::slice::from_raw_parts_mut(ptr, self.header.count())
    }

    /// Returns the full capacity slice (including unused bytes)
    ///
    /// # Safety
    ///
    /// The caller must ensure this array was properly allocated.
    #[inline]
    pub unsafe fn as_full_slice(&self) -> &[u8] {
        let ptr = (self as *const Self as *const u8).add(size_of::<JSByteArrayHeader>());
        core::slice::from_raw_parts(ptr, self.header.capacity())
    }

    /// Returns the mutable full capacity slice
    ///
    /// # Safety
    ///
    /// The caller must ensure this array was properly allocated.
    #[inline]
    pub unsafe fn as_full_mut_slice(&mut self) -> &mut [u8] {
        let ptr = (self as *mut Self as *mut u8).add(size_of::<JSByteArrayHeader>());
        core::slice::from_raw_parts_mut(ptr, self.header.capacity())
    }
}

// Helper function for size_of
const fn size_of<T>() -> usize {
    core::mem::size_of::<T>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_array_header() {
        let mut header = JSValueArrayHeader::new(10);
        assert_eq!(header.count(), 0);
        assert_eq!(header.capacity(), 10);

        header.set_count(5);
        assert_eq!(header.count(), 5);
    }

    #[test]
    fn test_byte_array_header() {
        let mut header = JSByteArrayHeader::new(100);
        assert_eq!(header.count(), 0);
        assert_eq!(header.capacity(), 100);

        header.set_count(50);
        assert_eq!(header.count(), 50);
    }

    #[test]
    fn test_value_array_alloc_size() {
        let size = JSValueArray::alloc_size(10);
        // Header (8 bytes) + 10 JSValues (8 bytes each on 64-bit)
        let expected = 8 + 10 * core::mem::size_of::<JSValue>();
        assert_eq!(size, expected);
    }

    #[test]
    fn test_byte_array_alloc_size() {
        let size = JSByteArray::alloc_size(100);
        // Header (8 bytes) + 100 bytes
        assert_eq!(size, 8 + 100);
    }

    #[test]
    fn test_header_sizes() {
        // Both headers are 8 bytes (two u32 fields)
        assert_eq!(core::mem::size_of::<JSValueArrayHeader>(), 8);
        assert_eq!(core::mem::size_of::<JSByteArrayHeader>(), 8);
    }
}
