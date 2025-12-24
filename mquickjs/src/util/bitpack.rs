//! Bit manipulation helpers

/// Extracts bits from a value
#[inline]
pub fn extract_bits(value: u32, shift: u32, mask: u32) -> u32 {
    (value >> shift) & mask
}

/// Sets bits in a value
#[inline]
pub fn set_bits(value: u32, new_bits: u32, shift: u32, mask: u32) -> u32 {
    (value & !(mask << shift)) | ((new_bits & mask) << shift)
}

/// Checks if a bit is set
#[inline]
pub fn is_bit_set(value: u32, bit: u32) -> bool {
    (value & (1 << bit)) != 0
}

/// Sets a bit
#[inline]
pub fn set_bit(value: u32, bit: u32) -> u32 {
    value | (1 << bit)
}

/// Clears a bit
#[inline]
pub fn clear_bit(value: u32, bit: u32) -> u32 {
    value & !(1 << bit)
}
