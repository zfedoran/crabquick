//! Garbage collector implementation
//!
//! Implements a mark-and-compact garbage collector using tri-color marking
//! and index-based compaction (safer than pointer threading).

use super::allocator::{Arena, HeapIndex};
use crate::value::JSValue;
use alloc::collections::BTreeMap as HashMap;
use alloc::vec::Vec;

/// Garbage collector state
pub struct GarbageCollector {
    /// Mark stack for tri-color marking (gray objects)
    mark_stack: Vec<HeapIndex>,
    /// Set of marked indices (for efficient lookup during compaction)
    /// We use this to track which indices are live during sweep/compact
    marked_indices: Vec<HeapIndex>,
    /// GC roots that must be preserved
    roots: Vec<JSValue>,
}

impl GarbageCollector {
    /// Creates a new garbage collector
    pub fn new() -> Self {
        GarbageCollector {
            mark_stack: Vec::new(),
            marked_indices: Vec::new(),
            roots: Vec::new(),
        }
    }

    /// Adds a root value that should be preserved during GC
    pub fn add_root(&mut self, value: JSValue) {
        self.roots.push(value);
    }

    /// Removes a root value
    pub fn remove_root(&mut self, value: JSValue) {
        // Compare raw bits by converting to usize
        let value_bits = value.as_raw();
        if let Some(pos) = self.roots.iter().position(|v: &JSValue| v.as_raw() == value_bits) {
            self.roots.swap_remove(pos);
        }
    }

    /// Performs a full garbage collection cycle
    ///
    /// Steps:
    /// 1. Mark roots
    /// 2. Mark reachable objects (tri-color marking)
    /// 3. Compact live objects and update index table
    pub fn collect(&mut self, arena: &mut Arena) {
        // Clear previous GC state
        self.mark_stack.clear();
        self.marked_indices.clear();

        // Phase 1: Mark all reachable objects
        self.mark_roots(arena);
        self.mark_phase(arena);

        // Phase 2: Compact live objects
        // This also implicitly sweeps dead objects
        self.compact(arena);
    }

    /// Marks all root objects
    fn mark_roots(&mut self, arena: &mut Arena) {
        // Mark all explicitly registered roots
        // Clone to avoid borrow conflict
        let roots: Vec<JSValue> = self.roots.clone();
        for root_value in roots {
            self.mark_value(root_value, arena);
        }

        // TODO: In a full implementation, also mark:
        // - Global object
        // - Exception value
        // - VM stack
        // - Current function call frames
    }

    /// Marks a JSValue if it's a pointer to a heap object
    fn mark_value(&mut self, value: JSValue, arena: &mut Arena) {
        if let Some(index) = value.to_ptr() {
            self.mark_object(index, arena);
        }
        // Non-pointer values (integers, special values) don't need marking
    }

    /// Marks an object and pushes it onto the mark stack
    fn mark_object(&mut self, index: HeapIndex, arena: &mut Arena) {
        if index.is_null() {
            return;
        }

        // Check if the index is valid
        if !arena.is_index_valid(index) {
            return;
        }

        // SAFETY: index should be valid as it came from a JSValue
        unsafe {
            let header = arena.get_header_mut(index);

            // Check if already marked (avoid cycles)
            if header.gc_mark() {
                return;
            }

            // Mark this object
            header.set_gc_mark(true);

            // Track this index as marked
            self.marked_indices.push(index);

            // Push onto mark stack for processing
            self.mark_stack.push(index);
        }
    }

    /// Processes the mark stack (tri-color marking)
    fn mark_phase(&mut self, arena: &mut Arena) {
        while let Some(index) = self.mark_stack.pop() {
            // Process this object's references
            self.scan_object(index, arena);
        }
    }

    /// Scans an object for references and marks them
    fn scan_object(&mut self, index: HeapIndex, arena: &mut Arena) {
        use super::header::MemTag;

        // SAFETY: index is from mark_stack, which only contains valid indices
        unsafe {
            let header = arena.get_header(index);
            let mtag = header.mtag();

            match mtag {
                MemTag::Object => {
                    // Scan JSObject fields - extract data first to avoid borrow conflicts
                    let obj: &crate::object::JSObject = arena.get(index);
                    let prototype = obj.prototype();
                    let has_props = obj.has_properties();
                    let props_idx = if has_props { Some(obj.props_index()) } else { None };
                    let has_class = obj.has_class_data();
                    let class_idx = if has_class { Some(obj.class_data_index()) } else { None };

                    // Now mark the extracted values
                    self.mark_value(prototype, arena);
                    if let Some(idx) = props_idx {
                        self.mark_object(idx, arena);
                    }
                    if let Some(idx) = class_idx {
                        self.mark_object(idx, arena);
                    }
                }
                MemTag::PropertyTable => {
                    // Scan property table - extract values first
                    let table: &crate::object::PropertyTable = arena.get(index);
                    let properties = table.properties();
                    let values_to_mark: Vec<(JSValue, Option<JSValue>)> = properties.iter()
                        .map(|prop| {
                            let setter = if prop.flags().has_set() { Some(prop.setter()) } else { None };
                            (prop.value(), setter)
                        })
                        .collect();

                    // Now mark the extracted values
                    for (value, setter) in values_to_mark {
                        self.mark_value(value, arena);
                        if let Some(s) = setter {
                            self.mark_value(s, arena);
                        }
                    }
                }
                MemTag::ValueArray => {
                    // Scan value array - extract values first
                    let array: &crate::value::JSValueArray = arena.get(index);
                    let values: Vec<JSValue> = array.as_slice().to_vec();

                    // Now mark the extracted values
                    for value in values {
                        self.mark_value(value, arena);
                    }
                }
                MemTag::String | MemTag::Float64 | MemTag::ByteArray => {
                    // These are leaf objects with no references
                }
                MemTag::FunctionBytecode => {
                    // TODO: Mark function bytecode references when implemented
                }
                MemTag::ClosureData => {
                    // Scan closure - mark all captured variable references
                    let closure: &crate::object::function::JSClosure = arena.get(index);
                    let var_ref_count = closure.var_ref_count as usize;

                    // Collect var ref indices first to avoid borrow conflicts
                    let var_refs: Vec<HeapIndex> = (0..var_ref_count)
                        .map(|i| closure.get_var_ref(i))
                        .collect();

                    // Mark all var refs
                    for vr_idx in var_refs {
                        self.mark_object(vr_idx, arena);
                    }
                }
                MemTag::VarRef => {
                    // Scan var ref - mark the contained value
                    let var_ref: &crate::object::function::JSVarRef = arena.get(index);
                    let value = var_ref.value();
                    self.mark_value(value, arena);
                }
                MemTag::CFunctionData => {
                    // C functions don't have GC references
                }
            }
        }
    }

    /// Compacts live objects using index-based approach
    ///
    /// This is the key simplification: we iterate through all indices,
    /// move live objects to compact memory, and update the index table.
    /// No need to thread pointers through objects!
    fn compact(&mut self, arena: &mut Arena) {
        let mut write_offset = 0;

        // Sort marked indices to process them in order
        // This isn't strictly necessary but makes the compaction more predictable
        self.marked_indices.sort_unstable();

        // Create a set of marked indices for O(log n) lookup
        let marked_set: HashMap<HeapIndex, ()> = self.marked_indices
            .iter()
            .map(|&idx| (idx, ()))
            .collect();

        // Iterate through all indices in the index table
        let index_count = arena.index_table_len();

        for idx in 0..index_count {
            let index = HeapIndex::from_usize(idx);

            // Get the current offset for this index
            let old_offset = match arena.get_offset(index) {
                Some(offset) => offset,
                None => continue, // Already freed, skip
            };

            // Check if this object is marked (live)
            let is_marked = marked_set.contains_key(&index);

            if is_marked {
                // Live object - move it to the compacted region
                unsafe {
                    let size = arena.get_block_size(old_offset);

                    // Only move if the object isn't already at the target location
                    if write_offset != old_offset {
                        // Move the object (header + data)
                        let src = arena.as_ptr().add(old_offset);
                        let dst = arena.as_mut_ptr().add(write_offset);
                        core::ptr::copy(src, dst, size);
                    }

                    // Update the index table to point to the new location
                    arena.update_index_offset(index, write_offset);

                    // Clear the mark bit for next GC cycle
                    let header = arena.get_header_mut(index);
                    header.set_gc_mark(false);

                    write_offset += size;
                }
            } else {
                // Dead object - free the index
                unsafe {
                    arena.free_index(index);
                }
            }
        }

        // Update the heap free pointer to the end of the compacted region
        unsafe {
            arena.set_heap_free(write_offset);
        }
    }
}

impl Default for GarbageCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemTag;

    #[test]
    fn test_gc_new() {
        let gc = GarbageCollector::new();
        // Just verify it constructs without panicking
        assert_eq!(gc.roots.len(), 0);
    }

    #[test]
    fn test_gc_roots() {
        let mut gc = GarbageCollector::new();

        let idx = HeapIndex::from_usize(0);
        let val = JSValue::from_ptr(idx);

        gc.add_root(val);
        assert_eq!(gc.roots.len(), 1);

        gc.remove_root(val);
        assert_eq!(gc.roots.len(), 0);
    }

    #[test]
    fn test_gc_mark_and_sweep() {
        let mut arena = Arena::new(1024);
        let mut gc = GarbageCollector::new();

        // Allocate some objects
        let idx1 = arena.alloc(64, MemTag::Object).unwrap();
        let idx2 = arena.alloc(128, MemTag::String).unwrap();

        // Root the first object
        let val1 = JSValue::from_ptr(idx1);
        gc.add_root(val1);

        // Run GC
        gc.collect(&mut arena);

        // First object should be marked during GC
        // (mark bits are cleared during sweep, so we can't check them after)
        // This test mainly verifies GC doesn't crash
    }

    #[test]
    fn test_gc_multiple_allocations() {
        let mut arena = Arena::new(4096);
        let mut gc = GarbageCollector::new();

        // Allocate many objects
        let mut indices = Vec::new();
        for _ in 0..10 {
            let idx = arena.alloc(32, MemTag::Object).unwrap();
            indices.push(idx);
        }

        // Root some of them
        for i in 0..5 {
            let val = JSValue::from_ptr(indices[i]);
            gc.add_root(val);
        }

        // Run GC
        gc.collect(&mut arena);

        // Verify no crashes
    }
}
