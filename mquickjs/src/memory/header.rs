//! Memory block headers with metadata
//!
//! Each allocated block has a header containing GC mark bits and type tags.

/// Memory tag identifying the type of allocated object
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MemTag {
    /// Generic object
    Object = 0,
    /// String
    String = 1,
    /// Float64 (boxed)
    Float64 = 2,
    /// Function bytecode
    FunctionBytecode = 3,
    /// Value array (JSValue[])
    ValueArray = 4,
    /// Byte array (u8[])
    ByteArray = 5,
    /// Property table
    PropertyTable = 6,
    /// Closure data
    ClosureData = 7,
    /// Variable reference
    VarRef = 8,
    /// C function data
    CFunctionData = 9,
    // TODO: Add more tags as needed
}

/// Memory block header
///
/// Packed into a u32:
/// - Bits 0-3: Memory tag (MemTag, values 0-15)
/// - Bit 4: GC mark bit
/// - Bits 5-31: Reserved for future use (size, flags, etc.)
///
/// Note: The actual size of the allocation is stored separately
/// in the allocator's metadata or can be tracked externally.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MemBlockHeader {
    data: u32,
    /// Size of the allocation in bytes (including header)
    /// This is stored separately to avoid bit-packing complexity
    size: u32,
}

impl MemBlockHeader {
    const MTAG_MASK: u32 = 0xF;  // 4 bits for mtag (0-15)
    const GC_MARK_BIT: u32 = 1 << 4;  // Move GC mark bit to bit 4

    /// Creates a new header with the specified memory tag and size
    pub fn new(mtag: MemTag, size: usize) -> Self {
        MemBlockHeader {
            data: mtag as u32,
            size: size as u32,
        }
    }

    /// Returns the size of the allocation (including header)
    #[inline]
    pub fn size(self) -> usize {
        self.size as usize
    }

    /// Sets the size of the allocation
    #[inline]
    pub fn set_size(&mut self, size: usize) {
        self.size = size as u32;
    }

    /// Returns the memory tag
    #[inline]
    pub fn mtag(self) -> MemTag {
        // TODO: Add proper error handling for invalid tags
        unsafe { core::mem::transmute((self.data & Self::MTAG_MASK) as u8) }
    }

    /// Sets the memory tag
    #[inline]
    pub fn set_mtag(&mut self, mtag: MemTag) {
        self.data = (self.data & !Self::MTAG_MASK) | (mtag as u32);
    }

    /// Returns the GC mark bit
    #[inline]
    pub fn gc_mark(self) -> bool {
        (self.data & Self::GC_MARK_BIT) != 0
    }

    /// Sets the GC mark bit
    #[inline]
    pub fn set_gc_mark(&mut self, marked: bool) {
        if marked {
            self.data |= Self::GC_MARK_BIT;
        } else {
            self.data &= !Self::GC_MARK_BIT;
        }
    }

    /// Clears all flags
    #[inline]
    pub fn clear_flags(&mut self) {
        self.data &= Self::MTAG_MASK;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        // Header is now 8 bytes (two u32 fields)
        assert_eq!(core::mem::size_of::<MemBlockHeader>(), 8);
    }

    #[test]
    fn test_mtag() {
        let mut header = MemBlockHeader::new(MemTag::String, 64);
        assert_eq!(header.mtag(), MemTag::String);
        assert_eq!(header.size(), 64);

        header.set_mtag(MemTag::Object);
        assert_eq!(header.mtag(), MemTag::Object);
    }

    #[test]
    fn test_gc_mark() {
        let mut header = MemBlockHeader::new(MemTag::Object, 128);
        assert!(!header.gc_mark());

        header.set_gc_mark(true);
        assert!(header.gc_mark());
        assert_eq!(header.mtag(), MemTag::Object); // mtag should be preserved
        assert_eq!(header.size(), 128); // size should be preserved

        header.set_gc_mark(false);
        assert!(!header.gc_mark());
    }

    #[test]
    fn test_size() {
        let mut header = MemBlockHeader::new(MemTag::String, 256);
        assert_eq!(header.size(), 256);

        header.set_size(512);
        assert_eq!(header.size(), 512);
        assert_eq!(header.mtag(), MemTag::String); // mtag should be preserved
    }
}
