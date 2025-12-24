//! Property hash table implementation
//!
//! Properties are stored in a compact hash table using open addressing.
//! Small objects (< 8 properties) use linear search; larger objects use a hash table.

use crate::value::{JSValue, JSAtom};

/// Property flags
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PropertyFlags(u8);

impl PropertyFlags {
    // Flag bits
    const WRITABLE: u8 = 1 << 0;
    const ENUMERABLE: u8 = 1 << 1;
    const CONFIGURABLE: u8 = 1 << 2;
    const HAS_GET: u8 = 1 << 3;
    const HAS_SET: u8 = 1 << 4;
    const IS_VARREF: u8 = 1 << 5; // For closure variable references

    /// Creates default flags (writable, enumerable, configurable)
    #[inline]
    pub const fn default() -> Self {
        PropertyFlags(Self::WRITABLE | Self::ENUMERABLE | Self::CONFIGURABLE)
    }

    /// Creates empty flags (all false)
    #[inline]
    pub const fn empty() -> Self {
        PropertyFlags(0)
    }

    /// Returns true if the property is writable
    #[inline]
    pub const fn is_writable(&self) -> bool {
        (self.0 & Self::WRITABLE) != 0
    }

    /// Returns true if the property is enumerable
    #[inline]
    pub const fn is_enumerable(&self) -> bool {
        (self.0 & Self::ENUMERABLE) != 0
    }

    /// Returns true if the property is configurable
    #[inline]
    pub const fn is_configurable(&self) -> bool {
        (self.0 & Self::CONFIGURABLE) != 0
    }

    /// Returns true if the property has a getter
    #[inline]
    pub const fn has_get(&self) -> bool {
        (self.0 & Self::HAS_GET) != 0
    }

    /// Returns true if the property has a setter
    #[inline]
    pub const fn has_set(&self) -> bool {
        (self.0 & Self::HAS_SET) != 0
    }

    /// Returns true if this is a variable reference (for closures)
    #[inline]
    pub const fn is_varref(&self) -> bool {
        (self.0 & Self::IS_VARREF) != 0
    }

    /// Sets the writable flag
    #[inline]
    pub fn set_writable(&mut self, value: bool) {
        if value {
            self.0 |= Self::WRITABLE;
        } else {
            self.0 &= !Self::WRITABLE;
        }
    }

    /// Sets the enumerable flag
    #[inline]
    pub fn set_enumerable(&mut self, value: bool) {
        if value {
            self.0 |= Self::ENUMERABLE;
        } else {
            self.0 &= !Self::ENUMERABLE;
        }
    }

    /// Sets the configurable flag
    #[inline]
    pub fn set_configurable(&mut self, value: bool) {
        if value {
            self.0 |= Self::CONFIGURABLE;
        } else {
            self.0 &= !Self::CONFIGURABLE;
        }
    }

    /// Sets the has_get flag
    #[inline]
    pub fn set_has_get(&mut self, value: bool) {
        if value {
            self.0 |= Self::HAS_GET;
        } else {
            self.0 &= !Self::HAS_GET;
        }
    }

    /// Sets the has_set flag
    #[inline]
    pub fn set_has_set(&mut self, value: bool) {
        if value {
            self.0 |= Self::HAS_SET;
        } else {
            self.0 &= !Self::HAS_SET;
        }
    }

    /// Sets the varref flag
    #[inline]
    pub fn set_varref(&mut self, value: bool) {
        if value {
            self.0 |= Self::IS_VARREF;
        } else {
            self.0 &= !Self::IS_VARREF;
        }
    }

    /// Creates flags for a getter/setter property
    #[inline]
    pub const fn getset(has_get: bool, has_set: bool) -> Self {
        let mut flags = Self::ENUMERABLE | Self::CONFIGURABLE;
        if has_get {
            flags |= Self::HAS_GET;
        }
        if has_set {
            flags |= Self::HAS_SET;
        }
        PropertyFlags(flags)
    }
}

/// Property entry in property table
///
/// Layout:
/// - key: JSAtom (4 bytes) - Property name as interned atom
/// - value: JSValue (4/8 bytes) - Property value or getter function
/// - setter: JSValue (4/8 bytes) - Setter function if HAS_SET flag is set
/// - hash_next: u32 (4 bytes) - Index of next property in hash chain (or u32::MAX)
/// - flags: PropertyFlags (1 byte) - Property descriptor flags
///
/// For normal properties, only key, value, and flags are relevant.
/// For getters/setters, value holds the getter and setter holds the setter.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Property {
    key: JSAtom,
    value: JSValue,
    setter: JSValue, // Only used if HAS_SET flag is set
    hash_next: u32,  // Index of next property in hash chain (u32::MAX = none)
    flags: PropertyFlags,
    _padding: [u8; 3], // Padding for alignment
}

impl Property {
    /// Creates a new data property
    #[inline]
    pub fn new_data(key: JSAtom, value: JSValue, flags: PropertyFlags) -> Self {
        Property {
            key,
            value,
            setter: JSValue::undefined(),
            hash_next: u32::MAX,
            flags,
            _padding: [0; 3],
        }
    }

    /// Creates a new accessor property (getter/setter)
    #[inline]
    pub fn new_accessor(
        key: JSAtom,
        getter: JSValue,
        setter: JSValue,
        flags: PropertyFlags,
    ) -> Self {
        Property {
            key,
            value: getter,
            setter,
            hash_next: u32::MAX,
            flags,
            _padding: [0; 3],
        }
    }

    /// Returns the property key (atom)
    #[inline]
    pub fn key(&self) -> JSAtom {
        self.key
    }

    /// Returns the property value (or getter if accessor)
    #[inline]
    pub fn value(&self) -> JSValue {
        self.value
    }

    /// Returns the setter function (only valid if HAS_SET is set)
    #[inline]
    pub fn setter(&self) -> JSValue {
        self.setter
    }

    /// Sets the property value
    #[inline]
    pub fn set_value(&mut self, value: JSValue) {
        self.value = value;
    }

    /// Sets the setter function
    #[inline]
    pub fn set_setter(&mut self, setter: JSValue) {
        self.setter = setter;
    }

    /// Returns the property flags
    #[inline]
    pub fn flags(&self) -> PropertyFlags {
        self.flags
    }

    /// Sets the property flags
    #[inline]
    pub fn set_flags(&mut self, flags: PropertyFlags) {
        self.flags = flags;
    }

    /// Returns the next property index in hash chain
    #[inline]
    pub fn hash_next(&self) -> u32 {
        self.hash_next
    }

    /// Sets the next property index in hash chain
    #[inline]
    pub fn set_hash_next(&mut self, next: u32) {
        self.hash_next = next;
    }

    /// Returns true if this is an accessor property
    #[inline]
    pub fn is_accessor(&self) -> bool {
        self.flags.has_get() || self.flags.has_set()
    }

    /// Returns true if this is a data property
    #[inline]
    pub fn is_data(&self) -> bool {
        !self.is_accessor()
    }
}

/// Property table header
///
/// The property table uses a sorted array for small objects (< 8 properties)
/// and switches to a hash table for larger objects.
///
/// Layout in memory:
/// [PropertyTableHeader][hash_table if count >= 8][Property array]
///
/// For small objects (count < 8):
/// - Properties are stored in a simple array
/// - Lookup is linear search
///
/// For larger objects (count >= 8):
/// - hash_table is allocated (size = next power of 2 >= count)
/// - Properties are stored in array
/// - hash_table[hash & mask] = index of first property in chain
/// - Properties with same hash are linked via hash_next field
#[repr(C)]
pub struct PropertyTableHeader {
    count: u32,      // Number of properties
    capacity: u32,   // Allocated capacity
    hash_mask: u32,  // Hash table mask (size - 1), or 0 if no hash table
    _padding: u32,   // For alignment
}

impl PropertyTableHeader {
    /// Threshold for switching from linear to hash table
    const HASH_THRESHOLD: u32 = 8;

    /// Creates a new property table header
    #[inline]
    pub fn new(capacity: u32) -> Self {
        PropertyTableHeader {
            count: 0,
            capacity,
            hash_mask: 0,
            _padding: 0,
        }
    }

    /// Returns the number of properties
    #[inline]
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Returns the allocated capacity
    #[inline]
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Returns the hash mask (0 if no hash table)
    #[inline]
    pub fn hash_mask(&self) -> u32 {
        self.hash_mask
    }

    /// Returns true if a hash table is used
    #[inline]
    pub fn has_hash_table(&self) -> bool {
        self.hash_mask != 0
    }

    /// Sets the count
    #[inline]
    pub fn set_count(&mut self, count: u32) {
        self.count = count;
    }

    /// Sets the capacity
    #[inline]
    pub fn set_capacity(&mut self, capacity: u32) {
        self.capacity = capacity;
    }

    /// Sets the hash mask
    #[inline]
    pub fn set_hash_mask(&mut self, mask: u32) {
        self.hash_mask = mask;
    }

    /// Returns the size of the hash table (0 if no hash table)
    #[inline]
    pub fn hash_table_size(&self) -> u32 {
        if self.hash_mask == 0 {
            0
        } else {
            self.hash_mask + 1
        }
    }

    /// Returns the total allocation size for a property table
    ///
    /// This includes:
    /// - PropertyTableHeader
    /// - Hash table (if count >= threshold)
    /// - Property array
    pub fn allocation_size(capacity: u32) -> usize {
        let header_size = core::mem::size_of::<PropertyTableHeader>();
        let properties_size = capacity as usize * core::mem::size_of::<Property>();

        // Determine if hash table is needed
        let hash_table_size = if capacity >= Self::HASH_THRESHOLD {
            // Find next power of 2
            let mut size = Self::HASH_THRESHOLD;
            while size < capacity {
                size *= 2;
            }
            size as usize * core::mem::size_of::<u32>()
        } else {
            0
        };

        header_size + hash_table_size + properties_size
    }

    /// Calculates the hash mask for a given capacity
    #[inline]
    pub fn calculate_hash_mask(capacity: u32) -> u32 {
        if capacity < Self::HASH_THRESHOLD {
            return 0;
        }

        // Find next power of 2
        let mut size = Self::HASH_THRESHOLD;
        while size < capacity {
            size *= 2;
        }
        size - 1
    }
}

/// Property table (opaque type - actual layout managed by Context)
///
/// The property table is allocated as a single memory block with the layout:
/// [PropertyTableHeader][hash_table (optional)][Property array]
///
/// Access to the table is through Context methods that handle the layout.
pub struct PropertyTable {
    _private: (),
}

impl PropertyTable {
    /// Returns the header of the property table
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` points to a valid PropertyTableHeader.
    #[inline]
    pub unsafe fn header(&self) -> &PropertyTableHeader {
        &*(self as *const Self as *const PropertyTableHeader)
    }

    /// Returns a mutable reference to the header
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` points to a valid PropertyTableHeader.
    #[inline]
    pub unsafe fn header_mut(&mut self) -> &mut PropertyTableHeader {
        &mut *(self as *mut Self as *mut PropertyTableHeader)
    }

    /// Returns a pointer to the hash table (or null if no hash table)
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` points to a valid PropertyTable.
    #[inline]
    pub unsafe fn hash_table_ptr(&self) -> *const u32 {
        let header = self.header();
        if !header.has_hash_table() {
            return core::ptr::null();
        }

        let ptr = self as *const Self as *const u8;
        let offset = core::mem::size_of::<PropertyTableHeader>();
        ptr.add(offset) as *const u32
    }

    /// Returns a mutable pointer to the hash table
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` points to a valid PropertyTable.
    #[inline]
    pub unsafe fn hash_table_ptr_mut(&mut self) -> *mut u32 {
        let header = self.header();
        if !header.has_hash_table() {
            return core::ptr::null_mut();
        }

        let ptr = self as *mut Self as *mut u8;
        let offset = core::mem::size_of::<PropertyTableHeader>();
        ptr.add(offset) as *mut u32
    }

    /// Returns a pointer to the property array
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` points to a valid PropertyTable.
    #[inline]
    pub unsafe fn properties_ptr(&self) -> *const Property {
        let header = self.header();
        let ptr = self as *const Self as *const u8;
        let offset = core::mem::size_of::<PropertyTableHeader>()
            + (header.hash_table_size() as usize * core::mem::size_of::<u32>());
        ptr.add(offset) as *const Property
    }

    /// Returns a mutable pointer to the property array
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` points to a valid PropertyTable.
    #[inline]
    pub unsafe fn properties_ptr_mut(&mut self) -> *mut Property {
        let header = self.header();
        let ptr = self as *mut Self as *mut u8;
        let offset = core::mem::size_of::<PropertyTableHeader>()
            + (header.hash_table_size() as usize * core::mem::size_of::<u32>());
        ptr.add(offset) as *mut Property
    }

    /// Returns a slice of all properties
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` points to a valid PropertyTable.
    #[inline]
    pub unsafe fn properties(&self) -> &[Property] {
        let header = self.header();
        let ptr = self.properties_ptr();
        core::slice::from_raw_parts(ptr, header.count() as usize)
    }

    /// Returns a mutable slice of all properties
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` points to a valid PropertyTable.
    #[inline]
    pub unsafe fn properties_mut(&mut self) -> &mut [Property] {
        let header = self.header();
        let ptr = self.properties_ptr_mut();
        core::slice::from_raw_parts_mut(ptr, header.count() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_flags() {
        let flags = PropertyFlags::default();
        assert!(flags.is_writable());
        assert!(flags.is_enumerable());
        assert!(flags.is_configurable());
        assert!(!flags.has_get());
        assert!(!flags.has_set());

        let mut flags = PropertyFlags::empty();
        assert!(!flags.is_writable());
        flags.set_writable(true);
        assert!(flags.is_writable());
    }

    #[test]
    fn test_property_flags_getset() {
        let flags = PropertyFlags::getset(true, false);
        assert!(flags.has_get());
        assert!(!flags.has_set());
        assert!(flags.is_enumerable());
        assert!(flags.is_configurable());
        assert!(!flags.is_writable());
    }

    #[test]
    fn test_property_data() {
        let atom = JSAtom::from_id(42);
        let value = JSValue::from_int(123);
        let prop = Property::new_data(atom, value, PropertyFlags::default());

        assert_eq!(prop.key(), atom);
        assert_eq!(prop.value(), value);
        assert!(prop.is_data());
        assert!(!prop.is_accessor());
        assert_eq!(prop.hash_next(), u32::MAX);
    }

    #[test]
    fn test_property_accessor() {
        let atom = JSAtom::from_id(42);
        let getter = JSValue::from_int(1);
        let setter = JSValue::from_int(2);
        let flags = PropertyFlags::getset(true, true);
        let prop = Property::new_accessor(atom, getter, setter, flags);

        assert_eq!(prop.key(), atom);
        assert_eq!(prop.value(), getter);
        assert_eq!(prop.setter(), setter);
        assert!(!prop.is_data());
        assert!(prop.is_accessor());
    }

    #[test]
    fn test_property_table_header() {
        let header = PropertyTableHeader::new(16);
        assert_eq!(header.count(), 0);
        assert_eq!(header.capacity(), 16);
        assert_eq!(header.hash_mask(), 0);
        assert!(!header.has_hash_table());
    }

    #[test]
    fn test_property_table_allocation_size() {
        // Small table (no hash table)
        let size1 = PropertyTableHeader::allocation_size(4);
        let expected1 = core::mem::size_of::<PropertyTableHeader>()
            + 4 * core::mem::size_of::<Property>();
        assert_eq!(size1, expected1);

        // Large table (with hash table)
        let size2 = PropertyTableHeader::allocation_size(16);
        let expected2 = core::mem::size_of::<PropertyTableHeader>()
            + 16 * core::mem::size_of::<u32>() // Hash table size = 16 (next pow2 >= 16)
            + 16 * core::mem::size_of::<Property>();
        assert_eq!(size2, expected2);
    }

    #[test]
    fn test_calculate_hash_mask() {
        assert_eq!(PropertyTableHeader::calculate_hash_mask(4), 0); // < threshold
        assert_eq!(PropertyTableHeader::calculate_hash_mask(8), 7); // 8-1
        assert_eq!(PropertyTableHeader::calculate_hash_mask(10), 15); // 16-1
        assert_eq!(PropertyTableHeader::calculate_hash_mask(16), 15); // 16-1
        assert_eq!(PropertyTableHeader::calculate_hash_mask(17), 31); // 32-1
    }

    #[test]
    fn test_property_size() {
        // Ensure Property is reasonably sized
        let size = core::mem::size_of::<Property>();
        // Should be around 24-32 bytes depending on platform
        assert!(size <= 32, "Property size is {}", size);
    }
}
