//! Bump allocator for heap memory
//!
//! Implements a simple bump allocator where the heap grows upward from the
//! bottom of the arena and the stack grows downward from the top.

use super::header::MemTag;

/// Heap index for stable references to allocated objects
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HeapIndex(pub u32);

/// Memory arena with bump allocator
///
/// Layout:
/// ```text
/// [heap →              free space              ← stack]
/// ```
pub struct Arena {
    // TODO: Implement fields:
    // - memory: Vec<u8>
    // - heap_free: usize (next allocation offset)
    // - stack_bottom: usize (stack top offset)
    _placeholder: u8,
}

impl Arena {
    /// Creates a new arena with the specified size
    pub fn new(_size: usize) -> Self {
        // TODO: Allocate backing memory
        Arena {
            _placeholder: 0,
        }
    }

    /// Allocates memory from the heap
    ///
    /// # Arguments
    ///
    /// * `size` - Size in bytes (will be aligned to 8 bytes)
    /// * `mtag` - Memory tag identifying object type
    ///
    /// # Returns
    ///
    /// HeapIndex on success, or error if out of memory
    pub fn alloc(&mut self, _size: usize, _mtag: MemTag) -> Result<HeapIndex, OutOfMemory> {
        // TODO: Implement bump allocation with alignment
        Err(OutOfMemory)
    }

    /// Frees the last allocated block (optimization for temporary allocations)
    pub fn free_last(&mut self, _index: HeapIndex) {
        // TODO: Implement if block is at heap_free
    }

    /// Shrinks an allocation to a smaller size
    pub fn shrink(&mut self, _index: HeapIndex, _new_size: usize) {
        // TODO: Implement shrinking (only if last block)
    }

    /// Returns the current heap usage in bytes
    pub fn heap_usage(&self) -> usize {
        // TODO: Return heap_free
        0
    }

    /// Returns the total arena size in bytes
    pub fn size(&self) -> usize {
        // TODO: Return memory.len()
        0
    }
}

/// Out of memory error
#[derive(Debug, Clone, Copy)]
pub struct OutOfMemory;
