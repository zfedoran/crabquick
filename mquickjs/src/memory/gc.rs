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
    /// Forwarding table: old index -> new offset
    /// Used during compaction to update references
    forwarding_table: HashMap<HeapIndex, usize>,
    /// GC roots that must be preserved
    roots: Vec<JSValue>,
}

impl GarbageCollector {
    /// Creates a new garbage collector
    pub fn new() -> Self {
        GarbageCollector {
            mark_stack: Vec::new(),
            forwarding_table: HashMap::new(),
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
        if let Some(pos) = self.roots.iter().position(|&v| v.as_raw() == value_bits) {
            self.roots.swap_remove(pos);
        }
    }

    /// Performs a full garbage collection cycle
    ///
    /// Steps:
    /// 1. Mark roots
    /// 2. Mark reachable objects (tri-color marking)
    /// 3. Sweep unmarked objects
    /// 4. Compact live objects
    /// 5. Update references
    pub fn collect(&mut self, arena: &mut Arena) {
        // Clear previous GC state
        self.mark_stack.clear();
        self.forwarding_table.clear();

        // Phase 1: Mark all reachable objects
        self.mark_roots(arena);
        self.mark_phase(arena);

        // Phase 2: Sweep unmarked objects (just clear mark bits)
        self.sweep(arena);

        // Phase 3: Compact live objects
        self.compact(arena);
    }

    /// Marks all root objects
    fn mark_roots(&mut self, arena: &mut Arena) {
        // Mark all explicitly registered roots
        for &root_value in &self.roots {
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

        // SAFETY: index should be valid as it came from a JSValue
        unsafe {
            let header = arena.get_header_mut(index);

            // Check if already marked (avoid cycles)
            if header.gc_mark() {
                return;
            }

            // Mark this object
            header.set_gc_mark(true);

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
                    // Scan JSObject fields
                    let obj: &crate::object::JSObject = arena.get(index);

                    // Mark prototype
                    self.mark_value(obj.prototype(), arena);

                    // Mark property table
                    if obj.has_properties() {
                        self.mark_object(obj.props_index(), arena);
                    }

                    // Mark class data
                    if obj.has_class_data() {
                        self.mark_object(obj.class_data_index(), arena);
                    }
                }
                MemTag::PropertyTable => {
                    // Scan property table and mark all property values
                    let table: &crate::object::PropertyTable = arena.get(index);
                    let properties = table.properties();

                    for prop in properties {
                        // Mark property value (or getter function)
                        self.mark_value(prop.value(), arena);

                        // Mark setter function if present
                        if prop.flags().has_set() {
                            self.mark_value(prop.setter(), arena);
                        }
                    }
                }
                MemTag::ValueArray => {
                    // Scan value array and mark all elements
                    let array: &crate::value::JSValueArray = arena.get(index);
                    let values = array.as_slice();

                    for &value in values {
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
                    // TODO: Mark closure variable references when implemented
                }
                MemTag::VarRef => {
                    // TODO: Mark variable reference value when implemented
                }
                MemTag::CFunctionData => {
                    // C functions don't have GC references
                }
            }
        }
    }

    /// Sweeps unmarked objects and clears mark bits on live ones
    fn sweep(&mut self, arena: &mut Arena) {
        let heap_size = arena.heap_usage();
        let mut offset = 0;

        while offset < heap_size {
            // SAFETY: We're iterating through allocated memory
            unsafe {
                let index = HeapIndex::from_usize(offset);
                let header = arena.get_header_mut(index);
                let size = header.size();

                if header.gc_mark() {
                    // Live object - clear mark bit for next GC cycle
                    header.set_gc_mark(false);
                } else {
                    // Dead object - would call finalizer here if implemented
                    // For now, just leave it (will be compacted away)
                }

                offset += size;
            }
        }
    }

    /// Compacts live objects using index-based approach
    fn compact(&mut self, arena: &mut Arena) {
        // Phase 1: Build forwarding table
        // Calculate new addresses for all live objects
        self.build_forwarding_table(arena);

        // Phase 2: Update all references
        self.update_references(arena);

        // Phase 3: Move objects to new locations
        self.move_objects(arena);
    }

    /// Builds the forwarding table: old index -> new offset
    fn build_forwarding_table(&mut self, arena: &mut Arena) {
        let heap_size = arena.heap_usage();
        let mut old_offset = 0;
        let mut new_offset = 0;

        while old_offset < heap_size {
            // SAFETY: Iterating through allocated memory
            unsafe {
                let index = HeapIndex::from_usize(old_offset);
                let header = arena.get_header(index);
                let size = header.size();

                // Note: In sweep phase, we cleared mark bits on live objects
                // But we need a way to identify live vs dead objects.
                // For now, we'll consider all objects as live.
                // TODO: Track dead objects in sweep phase

                // Record forwarding address
                self.forwarding_table.insert(index, new_offset);

                old_offset += size;
                new_offset += size;
            }
        }
    }

    /// Updates all references using the forwarding table
    fn update_references(&mut self, arena: &mut Arena) {
        // Update root references
        for root_value in &mut self.roots {
            if let Some(old_index) = root_value.to_ptr() {
                if let Some(&new_offset) = self.forwarding_table.get(&old_index) {
                    *root_value = JSValue::from_ptr(HeapIndex::from_usize(new_offset));
                }
            }
        }

        // TODO: Update references within objects
        // This requires scanning each object and updating its pointer fields
        // We'll implement this when we have concrete object types
    }

    /// Moves objects to their new locations
    fn move_objects(&mut self, arena: &mut Arena) {
        // TODO: Implement actual object movement
        // This is complex and requires:
        // 1. A temporary buffer or careful ordering to avoid overwriting
        // 2. Copying each live object to its new location
        // 3. Updating arena.heap_free to the new compacted size
        //
        // For now, this is a placeholder
        // We'll implement this fully when we have a complete object system
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
