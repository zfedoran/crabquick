//! # CrabQuick - A Minimal JavaScript Engine in Rust
//!
//! CrabQuick is a native Rust port of MicroQuickJS, designed to run JavaScript
//! in extremely constrained environments (minimum 10-12 kB RAM).
//!
//! ## Architecture
//!
//! The engine is organized into several major subsystems:
//!
//! - **Memory Management**: Custom bump allocator with index-based GC
//! - **Value System**: NaN-boxed tagged value representation
//! - **Object System**: Property hash tables and prototype chains
//! - **Bytecode**: Instruction format and constant pools
//! - **Compiler**: Lexer, parser, and code generator
//! - **Virtual Machine**: Bytecode interpreter with stack management
//! - **Built-ins**: JavaScript standard library implementation
//! - **Runtime**: Type conversions and operator implementations
//!
//! ## Example
//!
//! ```rust,ignore
//! use crabquick::Context;
//!
//! let mut ctx = Context::new(8192);
//! let result = ctx.eval("2 + 2", "script.js", 0)?;
//! assert_eq!(result.to_int(), Some(4));
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

extern crate alloc;

// Public API exports
pub use context::Context;
pub use value::JSValue;
pub use engine::{Engine, MemoryStats};

// Module declarations
pub mod memory;
pub mod value;
pub mod object;
pub mod bytecode;
pub mod compiler;
pub mod vm;
pub mod builtins;
pub mod runtime;
pub mod util;
pub mod engine;

// Core types
mod context;

// Re-exports for convenience
pub mod prelude {
    //! Commonly used types and traits
    pub use crate::context::Context;
    pub use crate::value::JSValue;
    pub use crate::engine::{Engine, MemoryStats};
}
