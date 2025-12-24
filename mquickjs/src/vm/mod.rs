//! Virtual machine (bytecode interpreter)

pub mod interpreter;
pub mod call;
pub mod exception;
pub mod stack;

// Re-exports
pub use interpreter::VM;
pub use stack::Stack;
pub use exception::Exception;
