//! Bump allocator for heap memory
//!
//! Implements a simple bump allocator where the heap grows upward from the
//! bottom of the arena and the stack grows downward from the top.

use super::header::{MemBlockHeader, MemTag};
use alloc::vec::Vec;
use core::mem;

/// Heap index for stable references to allocated objects
///
/// HeapIndex represents an offset into the arena's memory.
/// Indices remain stable across garbage collection (they get updated
/// during compaction via a forwarding table).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct HeapIndex(pub u32);

impl HeapIndex {
    /// Creates a null/invalid heap index
    pub const fn null() -> Self {
        HeapIndex(u32::MAX)
    }

    /// Returns true if this is a null/invalid index
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == u32::MAX
    }

    /// Returns the raw offset value
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Creates a HeapIndex from a raw offset
    #[inline]
    pub const fn from_usize(offset: usize) -> Self {
        HeapIndex(offset as u32)
    }
}

/// Memory arena with bump allocator
///
/// Layout:
/// ```text
/// [heap →              free space              ← stack]
/// ```
///
/// The heap grows upward from offset 0, and the stack (if used) would grow
/// downward from the top. Currently stack is not implemented but space is reserved.
pub struct Arena {
    /// Backing memory for the arena
    memory: Vec<u8>,
    /// Next free offset for heap allocations (grows upward)
    heap_free: usize,
    /// Bottom of stack space (grows downward from arena end)
    stack_bottom: usize,
}

/// 8-byte alignment for all allocations
const ALIGNMENT: usize = 8;

/// Aligns a size up to the specified alignment
#[inline]
const fn align_up(size: usize, alignment: usize) -> usize {
    (size + alignment - 1) & !(alignment - 1)
}

impl Arena {
    /// Creates a new arena with the specified size
    pub fn new(size: usize) -> Self {
        let mut memory = Vec::with_capacity(size);
        // Initialize memory to zero
        memory.resize(size, 0);

        Arena {
            memory,
            heap_free: 0,
            stack_bottom: size,
        }
    }

    /// Allocates memory from the heap
    ///
    /// # Arguments
    ///
    /// * `size` - Size in bytes (will be aligned to 8 bytes, plus header)
    /// * `mtag` - Memory tag identifying object type
    ///
    /// # Returns
    ///
    /// HeapIndex on success, or error if out of memory
    ///
    /// # Safety
    ///
    /// The returned HeapIndex is valid until the next GC compaction.
    pub fn alloc(&mut self, size: usize, mtag: MemTag) -> Result<HeapIndex, OutOfMemory> {
        // Calculate total size: header + data, aligned to 8 bytes
        let header_size = mem::size_of::<MemBlockHeader>();
        let total_size = align_up(header_size + size, ALIGNMENT);

        // Check if we have enough space
        if self.heap_free + total_size > self.stack_bottom {
            return Err(OutOfMemory);
        }

        // Store the allocation offset
        let index = HeapIndex::from_usize(self.heap_free);

        // SAFETY: We checked bounds above, and heap_free is always valid
        unsafe {
            let ptr = self.memory.as_mut_ptr().add(self.heap_free);
            let header_ptr = ptr as *mut MemBlockHeader;

            // Initialize header
            header_ptr.write(MemBlockHeader::new(mtag, total_size));
        }

        // Bump the allocation pointer
        self.heap_free += total_size;

        Ok(index)
    }

    /// Frees the last allocated block (optimization for temporary allocations)
    ///
    /// This only works if the given index points to the most recently allocated block.
    /// Otherwise, this is a no-op (safe but ineffective).
    pub fn free_last(&mut self, index: HeapIndex) {
        if index.is_null() {
            return;
        }

        let offset = index.as_usize();

        // Check if this is indeed the last block
        if offset < self.heap_free {
            // SAFETY: offset is within bounds (< heap_free)
            let size = unsafe {
                let ptr = self.memory.as_ptr().add(offset);
                let header_ptr = ptr as *const MemBlockHeader;
                (*header_ptr).size()
            };

            // Only free if this is the last allocated block
            if offset + size == self.heap_free {
                self.heap_free = offset;
            }
        }
    }

    /// Shrinks an allocation to a smaller size
    ///
    /// This only works if:
    /// 1. The block is the most recently allocated one
    /// 2. new_size is smaller than current size
    ///
    /// Otherwise, this is a no-op.
    pub fn shrink(&mut self, index: HeapIndex, new_size: usize) {
        if index.is_null() {
            return;
        }

        let offset = index.as_usize();

        if offset >= self.heap_free {
            return;
        }

        // SAFETY: offset is within bounds
        unsafe {
            let ptr = self.memory.as_mut_ptr().add(offset);
            let header_ptr = ptr as *mut MemBlockHeader;
            let header = &mut *header_ptr;
            let old_size = header.size();

            // Only shrink if this is the last block and new size is smaller
            if offset + old_size == self.heap_free {
                let header_size = mem::size_of::<MemBlockHeader>();
                let new_total_size = align_up(header_size + new_size, ALIGNMENT);

                if new_total_size < old_size {
                    header.set_size(new_total_size);
                    self.heap_free = offset + new_total_size;
                }
            }
        }
    }

    /// Returns the current heap usage in bytes
    #[inline]
    pub fn heap_usage(&self) -> usize {
        self.heap_free
    }

    /// Returns the total arena size in bytes
    #[inline]
    pub fn size(&self) -> usize {
        self.memory.len()
    }

    /// Returns the amount of free space available
    #[inline]
    pub fn free_space(&self) -> usize {
        self.stack_bottom.saturating_sub(self.heap_free)
    }

    /// Gets a reference to an object at the given index
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. The index is valid (was returned by alloc and not freed)
    /// 2. The index points to an object of type T
    /// 3. No mutable reference to this object exists
    #[inline]
    pub unsafe fn get<T>(&self, index: HeapIndex) -> &T {
        let offset = index.as_usize();
        let header_size = mem::size_of::<MemBlockHeader>();
        let ptr = self.memory.as_ptr().add(offset + header_size);
        &*(ptr as *const T)
    }

    /// Gets a mutable reference to an object at the given index
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. The index is valid (was returned by alloc and not freed)
    /// 2. The index points to an object of type T
    /// 3. No other reference to this object exists
    #[inline]
    pub unsafe fn get_mut<T>(&mut self, index: HeapIndex) -> &mut T {
        let offset = index.as_usize();
        let header_size = mem::size_of::<MemBlockHeader>();
        let ptr = self.memory.as_mut_ptr().add(offset + header_size);
        &mut *(ptr as *mut T)
    }

    /// Gets a reference to the header at the given index
    ///
    /// # Safety
    ///
    /// The caller must ensure that the index is valid (was returned by alloc and not freed).
    #[inline]
    pub unsafe fn get_header(&self, index: HeapIndex) -> &MemBlockHeader {
        let offset = index.as_usize();
        let ptr = self.memory.as_ptr().add(offset);
        &*(ptr as *const MemBlockHeader)
    }

    /// Gets a mutable reference to the header at the given index
    ///
    /// # Safety
    ///
    /// The caller must ensure that the index is valid (was returned by alloc and not freed).
    #[inline]
    pub unsafe fn get_header_mut(&mut self, index: HeapIndex) -> &mut MemBlockHeader {
        let offset = index.as_usize();
        let ptr = self.memory.as_mut_ptr().add(offset);
        &mut *(ptr as *mut MemBlockHeader)
    }

    /// Returns a pointer to the raw memory buffer
    ///
    /// This is useful for GC operations that need to iterate over all blocks.
    ///
    /// # Safety
    ///
    /// The caller must ensure proper bounds checking when using this pointer.
    #[inline]
    pub unsafe fn as_ptr(&self) -> *const u8 {
        self.memory.as_ptr()
    }

    /// Returns a mutable pointer to the raw memory buffer
    ///
    /// # Safety
    ///
    /// The caller must ensure proper bounds checking and no aliasing when using this pointer.
    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
        self.memory.as_mut_ptr()
    }
}

/// Out of memory error
#[derive(Debug, Clone, Copy)]
pub struct OutOfMemory;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heap_index() {
        let idx = HeapIndex::from_usize(100);
        assert_eq!(idx.as_usize(), 100);
        assert!(!idx.is_null());

        let null_idx = HeapIndex::null();
        assert!(null_idx.is_null());
    }

    #[test]
    fn test_arena_new() {
        let arena = Arena::new(1024);
        assert_eq!(arena.size(), 1024);
        assert_eq!(arena.heap_usage(), 0);
        assert_eq!(arena.free_space(), 1024);
    }

    #[test]
    fn test_arena_alloc() {
        let mut arena = Arena::new(1024);

        // Allocate a block
        let idx1 = arena.alloc(16, MemTag::Object).unwrap();
        assert!(!idx1.is_null());
        assert_eq!(idx1.as_usize(), 0);

        // Check that heap_free increased
        let usage1 = arena.heap_usage();
        assert!(usage1 > 0);
        assert!(usage1 >= 16 + mem::size_of::<MemBlockHeader>());

        // Allocate another block
        let idx2 = arena.alloc(32, MemTag::String).unwrap();
        assert!(!idx2.is_null());
        assert!(idx2.as_usize() > idx1.as_usize());

        let usage2 = arena.heap_usage();
        assert!(usage2 > usage1);
    }

    #[test]
    fn test_arena_alloc_alignment() {
        let mut arena = Arena::new(1024);

        // Allocate various sizes and verify alignment
        for size in [1, 7, 8, 9, 15, 16, 17, 31, 32, 33] {
            let idx = arena.alloc(size, MemTag::Object).unwrap();
            assert_eq!(idx.as_usize() % ALIGNMENT, 0, "Allocation at size {} not aligned", size);
        }
    }

    #[test]
    fn test_arena_out_of_memory() {
        let mut arena = Arena::new(128);

        // Fill up the arena
        let mut allocations = Vec::new();
        loop {
            match arena.alloc(16, MemTag::Object) {
                Ok(idx) => allocations.push(idx),
                Err(OutOfMemory) => break,
            }
        }

        // Should have some allocations
        assert!(!allocations.is_empty());

        // Should be nearly full
        assert!(arena.free_space() < 32);
    }

    #[test]
    fn test_arena_free_last() {
        let mut arena = Arena::new(1024);

        let idx1 = arena.alloc(16, MemTag::Object).unwrap();
        let usage_after_first = arena.heap_usage();

        let idx2 = arena.alloc(32, MemTag::String).unwrap();
        let usage_after_second = arena.heap_usage();

        // Free the last allocation
        arena.free_last(idx2);
        assert_eq!(arena.heap_usage(), usage_after_first);

        // Free the first allocation (which is now last)
        arena.free_last(idx1);
        assert_eq!(arena.heap_usage(), 0);
    }

    #[test]
    fn test_arena_free_last_non_last() {
        let mut arena = Arena::new(1024);

        let idx1 = arena.alloc(16, MemTag::Object).unwrap();
        let idx2 = arena.alloc(32, MemTag::String).unwrap();
        let usage = arena.heap_usage();

        // Try to free first block (not last) - should be a no-op
        arena.free_last(idx1);
        assert_eq!(arena.heap_usage(), usage);

        // Free actually last block - should work
        arena.free_last(idx2);
        assert!(arena.heap_usage() < usage);
    }

    #[test]
    fn test_arena_shrink() {
        let mut arena = Arena::new(1024);

        let idx = arena.alloc(64, MemTag::Object).unwrap();
        let initial_usage = arena.heap_usage();

        // Shrink the allocation
        arena.shrink(idx, 32);
        let new_usage = arena.heap_usage();

        // Usage should have decreased
        assert!(new_usage < initial_usage);
    }

    #[test]
    fn test_arena_get_header() {
        let mut arena = Arena::new(1024);

        let idx = arena.alloc(64, MemTag::String).unwrap();

        unsafe {
            let header = arena.get_header(idx);
            assert_eq!(header.mtag(), MemTag::String);
            assert!(!header.gc_mark());
        }
    }

    #[test]
    fn test_arena_get_mut_header() {
        let mut arena = Arena::new(1024);

        let idx = arena.alloc(64, MemTag::Object).unwrap();

        unsafe {
            let header = arena.get_header_mut(idx);
            header.set_gc_mark(true);

            let header = arena.get_header(idx);
            assert!(header.gc_mark());
        }
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(1, 8), 8);
        assert_eq!(align_up(7, 8), 8);
        assert_eq!(align_up(8, 8), 8);
        assert_eq!(align_up(9, 8), 16);
        assert_eq!(align_up(16, 8), 16);
        assert_eq!(align_up(17, 8), 24);
    }

    #[test]
    fn test_arena_get_and_get_mut() {
        #[repr(C)]
        struct TestStruct {
            a: u32,
            b: u64,
        }

        let mut arena = Arena::new(1024);
        let idx = arena.alloc(mem::size_of::<TestStruct>(), MemTag::Object).unwrap();

        unsafe {
            let obj = arena.get_mut::<TestStruct>(idx);
            obj.a = 42;
            obj.b = 12345;

            let obj_ref = arena.get::<TestStruct>(idx);
            assert_eq!(obj_ref.a, 42);
            assert_eq!(obj_ref.b, 12345);
        }
    }
}
