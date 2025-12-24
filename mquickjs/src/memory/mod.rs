//! Memory management subsystem
//!
//! This module implements the custom bump allocator and mark-and-compact
//! garbage collector used by MicroQuickJS.
//!
//! ## Architecture
//!
//! - **Arena**: Bump allocator with heap growing upward and stack downward
//! - **GC**: Mark-and-compact garbage collector with pointer threading
//! - **Handles**: GC root handles for protecting values during allocation
//! - **Headers**: Memory block headers with metadata and GC mark bits

pub mod allocator;
pub mod gc;
pub mod handle;
pub mod header;

// Re-exports
pub use allocator::Arena;
pub use gc::GarbageCollector;
pub use handle::GcRoot;
pub use header::{MemBlockHeader, MemTag};
