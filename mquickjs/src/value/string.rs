//! JavaScript string implementation
//!
//! JSString stores UTF-8 encoded strings with flags for optimizations.
//! Strings can be:
//! - ASCII-only (for fast case conversion and length calculations)
//! - Numeric (for fast parseInt-style conversions)
//! - Interned (unique strings stored in atom table)

use core::fmt;
use core::str;

/// JavaScript string header
///
/// Layout in memory:
/// ```text
/// [MemBlockHeader][JSStringHeader][UTF-8 bytes...]
/// ```
///
/// The header stores flags and length, followed by the actual UTF-8 data.
#[repr(C)]
pub struct JSStringHeader {
    /// Packed flags and length:
    /// - Bit 0: is_ascii (true if all chars are ASCII)
    /// - Bit 1: is_numeric (true if string looks like a number)
    /// - Bit 2: hash_valid (true if hash has been computed)
    /// - Bits 3-31: length in bytes (UTF-8 byte length, max 2^29 bytes)
    flags_and_len: u32,
    /// Hash value (cached when computed)
    hash: u32,
}

impl JSStringHeader {
    const IS_ASCII_BIT: u32 = 1 << 0;
    const IS_NUMERIC_BIT: u32 = 1 << 1;
    const HASH_VALID_BIT: u32 = 1 << 2;
    const LEN_SHIFT: u32 = 3;
    const LEN_MASK: u32 = !0x7; // All bits except lower 3

    /// Creates a new string header
    pub fn new(len: usize, is_ascii: bool, is_numeric: bool) -> Self {
        let mut flags_and_len = (len as u32) << Self::LEN_SHIFT;
        if is_ascii {
            flags_and_len |= Self::IS_ASCII_BIT;
        }
        if is_numeric {
            flags_and_len |= Self::IS_NUMERIC_BIT;
        }

        JSStringHeader {
            flags_and_len,
            hash: 0,
        }
    }

    /// Returns the byte length of the string
    #[inline]
    pub fn len(&self) -> usize {
        (self.flags_and_len >> Self::LEN_SHIFT) as usize
    }

    /// Returns true if the string is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if all characters are ASCII
    #[inline]
    pub fn is_ascii(&self) -> bool {
        (self.flags_and_len & Self::IS_ASCII_BIT) != 0
    }

    /// Sets the ASCII flag
    #[inline]
    pub fn set_ascii(&mut self, is_ascii: bool) {
        if is_ascii {
            self.flags_and_len |= Self::IS_ASCII_BIT;
        } else {
            self.flags_and_len &= !Self::IS_ASCII_BIT;
        }
    }

    /// Returns true if the string looks numeric
    #[inline]
    pub fn is_numeric(&self) -> bool {
        (self.flags_and_len & Self::IS_NUMERIC_BIT) != 0
    }

    /// Sets the numeric flag
    #[inline]
    pub fn set_numeric(&mut self, is_numeric: bool) {
        if is_numeric {
            self.flags_and_len |= Self::IS_NUMERIC_BIT;
        } else {
            self.flags_and_len &= !Self::IS_NUMERIC_BIT;
        }
    }

    /// Returns true if hash has been computed
    #[inline]
    pub fn hash_valid(&self) -> bool {
        (self.flags_and_len & Self::HASH_VALID_BIT) != 0
    }

    /// Returns the cached hash value
    #[inline]
    pub fn hash(&self) -> u32 {
        self.hash
    }

    /// Sets the hash value
    #[inline]
    pub fn set_hash(&mut self, hash: u32) {
        self.hash = hash;
        self.flags_and_len |= Self::HASH_VALID_BIT;
    }

    /// Returns the size of the complete allocation (header + data)
    #[inline]
    pub fn alloc_size(&self) -> usize {
        size_of::<crate::memory::MemBlockHeader>() + size_of::<JSStringHeader>() + self.len()
    }
}

/// JavaScript string
///
/// A JSString is a heap-allocated UTF-8 string with metadata.
/// The actual struct is the header; the UTF-8 data follows immediately after.
pub struct JSString {
    header: JSStringHeader,
    // UTF-8 data follows here (flexible array member)
}

impl JSString {
    /// Returns the header size
    #[inline]
    pub const fn header_size() -> usize {
        size_of::<JSStringHeader>()
    }

    /// Returns the total size needed for a string allocation (excluding MemBlockHeader)
    #[inline]
    pub const fn alloc_size(byte_len: usize) -> usize {
        size_of::<JSStringHeader>() + byte_len
    }

    /// Returns the header
    #[inline]
    pub fn header(&self) -> &JSStringHeader {
        &self.header
    }

    /// Returns the mutable header
    #[inline]
    pub fn header_mut(&mut self) -> &mut JSStringHeader {
        &mut self.header
    }

    /// Returns the UTF-8 byte slice
    ///
    /// # Safety
    ///
    /// The caller must ensure this JSString was properly allocated with
    /// the correct size to hold the UTF-8 data.
    #[inline]
    pub unsafe fn as_bytes(&self) -> &[u8] {
        let ptr = (self as *const Self as *const u8).add(size_of::<JSStringHeader>());
        core::slice::from_raw_parts(ptr, self.header.len())
    }

    /// Returns the string as a &str
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// 1. This JSString was properly allocated
    /// 2. The UTF-8 data is valid
    #[inline]
    pub unsafe fn as_str(&self) -> &str {
        str::from_utf8_unchecked(self.as_bytes())
    }

    /// Computes the hash of this string
    ///
    /// Uses a simple FNV-1a hash for speed.
    pub fn compute_hash(&self) -> u32 {
        unsafe {
            let bytes = self.as_bytes();
            let mut hash: u32 = 2166136261; // FNV offset basis
            for &byte in bytes {
                hash ^= byte as u32;
                hash = hash.wrapping_mul(16777619); // FNV prime
            }
            hash
        }
    }

    /// Returns the hash, computing it if needed
    pub fn hash(&mut self) -> u32 {
        if !self.header.hash_valid() {
            let h = self.compute_hash();
            self.header.set_hash(h);
        }
        self.header.hash()
    }

    /// Returns the cached hash if available
    pub fn hash_cached(&self) -> Option<u32> {
        if self.header.hash_valid() {
            Some(self.header.hash())
        } else {
            None
        }
    }

    /// Checks if the string is ASCII-only
    pub fn check_ascii(bytes: &[u8]) -> bool {
        bytes.iter().all(|&b| b < 128)
    }

    /// Checks if the string looks numeric (simple check)
    pub fn check_numeric(bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return false;
        }

        // Simple check: starts with digit or '-', contains only digits and '.' and 'e'/'E'
        let mut has_digit = false;
        for (i, &b) in bytes.iter().enumerate() {
            match b {
                b'0'..=b'9' => has_digit = true,
                b'-' | b'+' => {
                    if i != 0 {
                        return false;
                    }
                }
                b'.' | b'e' | b'E' => {}
                _ => return false,
            }
        }
        has_digit
    }

    /// Returns the character count (slower for non-ASCII)
    pub fn char_count(&self) -> usize {
        unsafe {
            if self.header.is_ascii() {
                self.header.len()
            } else {
                self.as_str().chars().count()
            }
        }
    }
}

impl fmt::Debug for JSString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            f.debug_struct("JSString")
                .field("len", &self.header.len())
                .field("is_ascii", &self.header.is_ascii())
                .field("is_numeric", &self.header.is_numeric())
                .field("data", &self.as_str())
                .finish()
        }
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
    fn test_header_flags() {
        let header = JSStringHeader::new(10, true, false);
        assert_eq!(header.len(), 10);
        assert!(header.is_ascii());
        assert!(!header.is_numeric());
        assert!(!header.hash_valid());
    }

    #[test]
    fn test_header_hash() {
        let mut header = JSStringHeader::new(5, true, false);
        assert!(!header.hash_valid());

        header.set_hash(12345);
        assert!(header.hash_valid());
        assert_eq!(header.hash(), 12345);
    }

    #[test]
    fn test_check_ascii() {
        assert!(JSString::check_ascii(b"hello"));
        assert!(JSString::check_ascii(b"123"));
        assert!(!JSString::check_ascii("café".as_bytes()));
        assert!(!JSString::check_ascii("你好".as_bytes()));
    }

    #[test]
    fn test_check_numeric() {
        assert!(JSString::check_numeric(b"123"));
        assert!(JSString::check_numeric(b"-456"));
        assert!(JSString::check_numeric(b"3.14"));
        assert!(JSString::check_numeric(b"1.5e10"));
        assert!(!JSString::check_numeric(b""));
        assert!(!JSString::check_numeric(b"abc"));
        assert!(!JSString::check_numeric(b"12a"));
    }

    #[test]
    fn test_header_size() {
        // Header is 8 bytes (u32 + u32)
        assert_eq!(core::mem::size_of::<JSStringHeader>(), 8);
    }

    #[test]
    fn test_alloc_size() {
        let size = JSString::alloc_size(10);
        assert_eq!(size, 8 + 10); // header + data
    }
}
