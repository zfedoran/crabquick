//! Atom system for interned strings
//!
//! Atoms are interned strings used primarily as property names.
//! Each unique string is stored only once, saving memory and enabling
//! fast equality comparison by comparing atom IDs instead of string contents.

extern crate alloc;
use alloc::vec::Vec;
use core::fmt;

use crate::memory::HeapIndex;

/// Atom identifier
///
/// An atom is a reference to an interned string.
/// Atoms can be compared for equality by comparing their IDs.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct JSAtom(u32);

impl JSAtom {
    /// Creates a new atom from an ID
    #[inline]
    pub const fn from_id(id: u32) -> Self {
        JSAtom(id)
    }

    /// Returns the atom ID
    #[inline]
    pub const fn id(&self) -> u32 {
        self.0
    }

    /// Returns the null atom (invalid)
    #[inline]
    pub const fn null() -> Self {
        JSAtom(u32::MAX)
    }

    /// Returns true if this is the null atom
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.0 == u32::MAX
    }
}

/// Entry in the atom table
#[derive(Clone)]
struct AtomEntry {
    /// Heap index of the interned JSString
    string_index: HeapIndex,
    /// Cached hash value
    hash: u32,
    /// Reference count (for GC)
    ref_count: u32,
}

/// Atom table for string interning
///
/// The atom table maintains a sorted array of unique strings.
/// Strings are stored on the heap as JSString objects, and the table
/// stores heap indices along with cached hashes for fast lookup.
///
/// The table is kept sorted by hash (and then by string content for collisions)
/// to enable binary search.
pub struct AtomTable {
    /// Sorted array of atom entries
    entries: Vec<AtomEntry>,
}

impl AtomTable {
    /// Creates a new empty atom table
    pub fn new() -> Self {
        AtomTable {
            entries: Vec::new(),
        }
    }

    /// Creates a new atom table with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        AtomTable {
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Returns the number of atoms in the table
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the table is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Looks up an atom by string and hash
    ///
    /// Returns Some(atom) if found, None otherwise.
    ///
    /// # Safety
    ///
    /// The caller must provide a valid arena reference to access strings.
    pub unsafe fn lookup(
        &self,
        string_bytes: &[u8],
        hash: u32,
        arena: &crate::memory::Arena,
    ) -> Option<JSAtom> {
        // Binary search by hash
        let mut left = 0;
        let mut right = self.entries.len();

        while left < right {
            let mid = (left + right) / 2;
            let entry = &self.entries[mid];

            match entry.hash.cmp(&hash) {
                core::cmp::Ordering::Less => left = mid + 1,
                core::cmp::Ordering::Greater => right = mid,
                core::cmp::Ordering::Equal => {
                    // Hash matches, compare string contents
                    let string: &crate::value::JSString = arena.get(entry.string_index);
                    let stored_bytes = string.as_bytes();

                    if stored_bytes == string_bytes {
                        return Some(JSAtom::from_id(mid as u32));
                    }

                    // Hash collision, search nearby entries
                    // Search left
                    let mut i = mid;
                    while i > 0 {
                        i -= 1;
                        let e = &self.entries[i];
                        if e.hash != hash {
                            break;
                        }
                        let s: &crate::value::JSString = arena.get(e.string_index);
                        if s.as_bytes() == string_bytes {
                            return Some(JSAtom::from_id(i as u32));
                        }
                    }

                    // Search right
                    let mut i = mid + 1;
                    while i < self.entries.len() {
                        let e = &self.entries[i];
                        if e.hash != hash {
                            break;
                        }
                        let s: &crate::value::JSString = arena.get(e.string_index);
                        if s.as_bytes() == string_bytes {
                            return Some(JSAtom::from_id(i as u32));
                        }
                        i += 1;
                    }

                    return None;
                }
            }
        }

        None
    }

    /// Interns a string, returning its atom
    ///
    /// If the string already exists, returns the existing atom.
    /// Otherwise, adds it to the table and returns a new atom.
    ///
    /// # Safety
    ///
    /// The caller must provide a valid string index.
    pub unsafe fn intern(&mut self, string_index: HeapIndex, hash: u32) -> JSAtom {
        // Find insertion point by binary search
        let mut left = 0;
        let mut right = self.entries.len();

        while left < right {
            let mid = (left + right) / 2;
            let entry = &self.entries[mid];

            if entry.hash < hash {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        // Insert at position 'left'
        let entry = AtomEntry {
            string_index,
            hash,
            ref_count: 1,
        };

        self.entries.insert(left, entry);
        JSAtom::from_id(left as u32)
    }

    /// Gets the string index for an atom
    ///
    /// Returns None if the atom is invalid.
    #[inline]
    pub fn get_string_index(&self, atom: JSAtom) -> Option<HeapIndex> {
        if atom.is_null() {
            return None;
        }

        self.entries.get(atom.id() as usize).map(|e| e.string_index)
    }

    /// Increments the reference count for an atom
    pub fn add_ref(&mut self, atom: JSAtom) {
        if let Some(entry) = self.entries.get_mut(atom.id() as usize) {
            entry.ref_count = entry.ref_count.saturating_add(1);
        }
    }

    /// Decrements the reference count for an atom
    ///
    /// Returns true if the ref count reached zero (atom can be freed).
    pub fn remove_ref(&mut self, atom: JSAtom) -> bool {
        if let Some(entry) = self.entries.get_mut(atom.id() as usize) {
            entry.ref_count = entry.ref_count.saturating_sub(1);
            entry.ref_count == 0
        } else {
            false
        }
    }

    /// Removes an atom from the table
    ///
    /// This should only be called when the ref count is zero.
    pub fn remove(&mut self, atom: JSAtom) {
        if !atom.is_null() && (atom.id() as usize) < self.entries.len() {
            self.entries.remove(atom.id() as usize);
        }
    }

    /// Clears all atoms with zero ref count
    ///
    /// This is called during GC to clean up unused atoms.
    pub fn gc_sweep(&mut self) {
        self.entries.retain(|entry| entry.ref_count > 0);
    }

    /// Returns an iterator over all atoms
    pub fn iter(&self) -> impl Iterator<Item = (JSAtom, HeapIndex)> + '_ {
        self.entries
            .iter()
            .enumerate()
            .map(|(i, entry)| (JSAtom::from_id(i as u32), entry.string_index))
    }
}

impl Default for AtomTable {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for AtomTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomTable")
            .field("len", &self.len())
            .field("capacity", &self.entries.capacity())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_creation() {
        let atom = JSAtom::from_id(42);
        assert_eq!(atom.id(), 42);
        assert!(!atom.is_null());
    }

    #[test]
    fn test_atom_null() {
        let atom = JSAtom::null();
        assert!(atom.is_null());
        assert_eq!(atom.id(), u32::MAX);
    }

    #[test]
    fn test_atom_equality() {
        let atom1 = JSAtom::from_id(10);
        let atom2 = JSAtom::from_id(10);
        let atom3 = JSAtom::from_id(20);

        assert_eq!(atom1, atom2);
        assert_ne!(atom1, atom3);
    }

    #[test]
    fn test_atom_table_creation() {
        let table = AtomTable::new();
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn test_atom_table_capacity() {
        let table = AtomTable::with_capacity(100);
        assert_eq!(table.len(), 0);
        assert!(table.entries.capacity() >= 100);
    }

    #[test]
    fn test_atom_intern() {
        let mut table = AtomTable::new();
        let idx1 = HeapIndex::from_usize(0);
        let idx2 = HeapIndex::from_usize(8);

        unsafe {
            let atom1 = table.intern(idx1, 12345);
            assert_eq!(table.len(), 1);
            assert_eq!(atom1.id(), 0);

            let atom2 = table.intern(idx2, 67890);
            assert_eq!(table.len(), 2);
            assert_eq!(atom2.id(), 1);
        }
    }

    #[test]
    fn test_atom_get_string_index() {
        let mut table = AtomTable::new();
        let idx = HeapIndex::from_usize(100);

        unsafe {
            let atom = table.intern(idx, 12345);
            assert_eq!(table.get_string_index(atom), Some(idx));
        }

        let null_atom = JSAtom::null();
        assert_eq!(table.get_string_index(null_atom), None);
    }

    #[test]
    fn test_atom_ref_counting() {
        let mut table = AtomTable::new();
        let idx = HeapIndex::from_usize(0);

        unsafe {
            let atom = table.intern(idx, 12345);

            // Initial ref count is 1
            table.add_ref(atom);
            assert_eq!(table.entries[atom.id() as usize].ref_count, 2);

            // Remove ref
            assert!(!table.remove_ref(atom)); // Still has refs
            assert!(!table.remove_ref(atom)); // Now zero

            assert!(table.remove_ref(atom)); // Returns true when zero
        }
    }

    #[test]
    fn test_atom_gc_sweep() {
        let mut table = AtomTable::new();

        unsafe {
            let atom1 = table.intern(HeapIndex::from_usize(0), 111);
            let atom2 = table.intern(HeapIndex::from_usize(8), 222);
            let _atom3 = table.intern(HeapIndex::from_usize(16), 333);

            // Set ref counts
            table.entries[atom1.id() as usize].ref_count = 1;
            table.entries[atom2.id() as usize].ref_count = 0;
            table.entries[2].ref_count = 1;

            table.gc_sweep();

            // atom2 should be removed
            assert_eq!(table.len(), 2);
        }
    }
}
