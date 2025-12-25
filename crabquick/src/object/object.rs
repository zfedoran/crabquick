//! JavaScript object representation
//!
//! This module implements the core JSObject structure that represents
//! all JavaScript objects including plain objects, arrays, functions, etc.

use crate::memory::HeapIndex;
use crate::value::JSValue;
use super::class::JSClassID;

/// JavaScript object
///
/// Layout:
/// - header: u32 (packed: class_id (8 bits) + gc_mark (1 bit) + flags (23 bits))
/// - proto: JSValue - Prototype object
/// - props: HeapIndex - Index to PropertyTable (or null if no properties)
/// - class_data: HeapIndex - Index to class-specific data (or null)
///
/// The header packing allows for:
/// - class_id: 256 different object classes
/// - gc_mark: GC marking bit (managed by GC, not by object directly)
/// - flags: Extended flags for future use
///
/// Memory layout:
/// [MemBlockHeader][JSObject]
#[repr(C)]
pub struct JSObject {
    header: u32,
    proto: JSValue,
    props: HeapIndex,      // PropertyTable index
    class_data: HeapIndex, // Class-specific data index
}

impl JSObject {
    // Bit positions in header
    const CLASS_ID_SHIFT: u32 = 0;
    const CLASS_ID_MASK: u32 = 0xFF;
    const FLAGS_SHIFT: u32 = 8;
    const FLAGS_MASK: u32 = 0xFFFFFF00;

    // Object flags (stored in header bits 8-31)
    const FLAG_EXTENSIBLE: u32 = 1 << 8;  // Object is extensible (can add properties)
    const FLAG_SEALED: u32 = 1 << 9;      // Object is sealed (cannot add/delete properties)
    const FLAG_FROZEN: u32 = 1 << 10;     // Object is frozen (cannot modify)

    /// Creates a new object with the specified class ID
    #[inline]
    pub fn new(class_id: JSClassID, proto: JSValue) -> Self {
        let header = (class_id as u32) | Self::FLAG_EXTENSIBLE;
        JSObject {
            header,
            proto,
            props: HeapIndex::null(),
            class_data: HeapIndex::null(),
        }
    }

    /// Creates a new plain object (with Object class)
    #[inline]
    pub fn new_plain(proto: JSValue) -> Self {
        Self::new(JSClassID::Object, proto)
    }

    /// Returns the class ID
    #[inline]
    pub fn class_id(&self) -> JSClassID {
        let id = (self.header & Self::CLASS_ID_MASK) as u8;
        // SAFETY: We only create objects with valid class IDs
        JSClassID::from_u8(id).unwrap_or(JSClassID::Object)
    }

    /// Sets the class ID
    #[inline]
    pub fn set_class_id(&mut self, class_id: JSClassID) {
        self.header = (self.header & Self::FLAGS_MASK) | (class_id as u32);
    }

    /// Returns the prototype
    #[inline]
    pub fn prototype(&self) -> JSValue {
        self.proto
    }

    /// Sets the prototype
    #[inline]
    pub fn set_prototype(&mut self, proto: JSValue) {
        self.proto = proto;
    }

    /// Returns the property table index
    #[inline]
    pub fn props_index(&self) -> HeapIndex {
        self.props
    }

    /// Sets the property table index
    #[inline]
    pub fn set_props_index(&mut self, index: HeapIndex) {
        self.props = index;
    }

    /// Returns true if the object has a property table
    #[inline]
    pub fn has_properties(&self) -> bool {
        !self.props.is_null()
    }

    /// Returns the class data index
    #[inline]
    pub fn class_data_index(&self) -> HeapIndex {
        self.class_data
    }

    /// Sets the class data index
    #[inline]
    pub fn set_class_data_index(&mut self, index: HeapIndex) {
        self.class_data = index;
    }

    /// Returns true if the object has class-specific data
    #[inline]
    pub fn has_class_data(&self) -> bool {
        !self.class_data.is_null()
    }

    /// Returns true if the object is extensible
    #[inline]
    pub fn is_extensible(&self) -> bool {
        (self.header & Self::FLAG_EXTENSIBLE) != 0
    }

    /// Sets the extensible flag
    #[inline]
    pub fn set_extensible(&mut self, extensible: bool) {
        if extensible {
            self.header |= Self::FLAG_EXTENSIBLE;
        } else {
            self.header &= !Self::FLAG_EXTENSIBLE;
        }
    }

    /// Returns true if the object is sealed
    #[inline]
    pub fn is_sealed(&self) -> bool {
        (self.header & Self::FLAG_SEALED) != 0
    }

    /// Seals the object (prevents adding/removing properties)
    #[inline]
    pub fn seal(&mut self) {
        self.header |= Self::FLAG_SEALED;
        self.header &= !Self::FLAG_EXTENSIBLE;
    }

    /// Returns true if the object is frozen
    #[inline]
    pub fn is_frozen(&self) -> bool {
        (self.header & Self::FLAG_FROZEN) != 0
    }

    /// Freezes the object (prevents all modifications)
    #[inline]
    pub fn freeze(&mut self) {
        self.header |= Self::FLAG_FROZEN | Self::FLAG_SEALED;
        self.header &= !Self::FLAG_EXTENSIBLE;
    }

    /// Returns true if this is a plain object
    #[inline]
    pub fn is_plain_object(&self) -> bool {
        self.class_id() == JSClassID::Object
    }

    /// Returns true if this is an array object
    #[inline]
    pub fn is_array(&self) -> bool {
        self.class_id() == JSClassID::Array
    }

    /// Returns true if this is a function object
    #[inline]
    pub fn is_function(&self) -> bool {
        self.class_id() == JSClassID::Function
    }

    /// Returns true if this is an error object
    #[inline]
    pub fn is_error(&self) -> bool {
        self.class_id() == JSClassID::Error
    }

    /// Returns true if this is a typed array object
    #[inline]
    pub fn is_typed_array(&self) -> bool {
        self.class_id().is_typed_array()
    }
}

/// Array-specific data
///
/// Stored in class_data for Array objects.
/// Arrays use a combination of a dense element array and property table
/// for non-index properties.
#[repr(C)]
pub struct JSArrayData {
    /// Dense array of elements (JSValueArray index)
    /// For arrays with sequential integer indices starting from 0.
    elements: HeapIndex,
    /// Logical length of the array
    length: u32,
    _padding: u32,
}

impl JSArrayData {
    /// Creates new array data
    #[inline]
    pub fn new(elements: HeapIndex, length: u32) -> Self {
        JSArrayData {
            elements,
            length,
            _padding: 0,
        }
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

    /// Returns the array length
    #[inline]
    pub fn length(&self) -> u32 {
        self.length
    }

    /// Sets the array length
    #[inline]
    pub fn set_length(&mut self, length: u32) {
        self.length = length;
    }

    /// Returns true if the array has an elements array
    #[inline]
    pub fn has_elements(&self) -> bool {
        !self.elements.is_null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_new() {
        let obj = JSObject::new_plain(JSValue::null());
        assert_eq!(obj.class_id(), JSClassID::Object);
        assert_eq!(obj.prototype(), JSValue::null());
        assert!(!obj.has_properties());
        assert!(!obj.has_class_data());
        assert!(obj.is_extensible());
        assert!(!obj.is_sealed());
        assert!(!obj.is_frozen());
    }

    #[test]
    fn test_object_class_id() {
        let mut obj = JSObject::new(JSClassID::Array, JSValue::null());
        assert_eq!(obj.class_id(), JSClassID::Array);
        assert!(obj.is_array());
        assert!(!obj.is_plain_object());

        obj.set_class_id(JSClassID::Function);
        assert_eq!(obj.class_id(), JSClassID::Function);
        assert!(obj.is_function());
    }

    #[test]
    fn test_object_prototype() {
        let mut obj = JSObject::new_plain(JSValue::null());
        assert_eq!(obj.prototype(), JSValue::null());

        let proto = JSValue::from_int(42);
        obj.set_prototype(proto);
        assert_eq!(obj.prototype(), proto);
    }

    #[test]
    fn test_object_properties() {
        let mut obj = JSObject::new_plain(JSValue::null());
        assert!(!obj.has_properties());
        assert!(obj.props_index().is_null());

        let props_idx = HeapIndex::from_usize(100);
        obj.set_props_index(props_idx);
        assert!(obj.has_properties());
        assert_eq!(obj.props_index(), props_idx);
    }

    #[test]
    fn test_object_class_data() {
        let mut obj = JSObject::new_plain(JSValue::null());
        assert!(!obj.has_class_data());

        let data_idx = HeapIndex::from_usize(200);
        obj.set_class_data_index(data_idx);
        assert!(obj.has_class_data());
        assert_eq!(obj.class_data_index(), data_idx);
    }

    #[test]
    fn test_object_extensible() {
        let mut obj = JSObject::new_plain(JSValue::null());
        assert!(obj.is_extensible());

        obj.set_extensible(false);
        assert!(!obj.is_extensible());

        obj.set_extensible(true);
        assert!(obj.is_extensible());
    }

    #[test]
    fn test_object_seal() {
        let mut obj = JSObject::new_plain(JSValue::null());
        assert!(!obj.is_sealed());
        assert!(obj.is_extensible());

        obj.seal();
        assert!(obj.is_sealed());
        assert!(!obj.is_extensible());
        assert!(!obj.is_frozen());
    }

    #[test]
    fn test_object_freeze() {
        let mut obj = JSObject::new_plain(JSValue::null());
        assert!(!obj.is_frozen());

        obj.freeze();
        assert!(obj.is_frozen());
        assert!(obj.is_sealed());
        assert!(!obj.is_extensible());
    }

    #[test]
    fn test_object_size() {
        // Verify object size is reasonable
        let size = core::mem::size_of::<JSObject>();
        // Should be around 16-24 bytes
        assert!(size <= 32, "JSObject size is {}", size);
    }

    #[test]
    fn test_array_data() {
        let elements = HeapIndex::from_usize(100);
        let mut array_data = JSArrayData::new(elements, 42);

        assert_eq!(array_data.elements_index(), elements);
        assert_eq!(array_data.length(), 42);
        assert!(array_data.has_elements());

        array_data.set_length(100);
        assert_eq!(array_data.length(), 100);

        let new_elements = HeapIndex::from_usize(200);
        array_data.set_elements_index(new_elements);
        assert_eq!(array_data.elements_index(), new_elements);
    }

    #[test]
    fn test_array_data_no_elements() {
        let array_data = JSArrayData::new(HeapIndex::null(), 0);
        assert!(!array_data.has_elements());
        assert_eq!(array_data.length(), 0);
    }
}
