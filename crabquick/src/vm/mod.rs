//! Virtual machine (bytecode interpreter)
//!
//! This module implements the JavaScript bytecode interpreter including:
//! - Value stack and call stack management
//! - Bytecode execution loop
//! - Opcode handlers for all instructions
//! - Exception handling
//! - Function call mechanics

pub mod interpreter;
pub mod call;
pub mod exception;
pub mod stack;

// Re-exports
pub use interpreter::VM;
pub use stack::{ValueStack, CallStack, StackFrame};
pub use stack::{StackOverflow, StackUnderflow, CallStackOverflow, CallStackUnderflow};
pub use exception::VMException;
