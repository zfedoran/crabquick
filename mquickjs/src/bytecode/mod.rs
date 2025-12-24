//! Bytecode format and instruction encoding
//!
//! Defines opcodes, instruction encoding, and constant pool management.

pub mod opcode;
pub mod format;
pub mod constants;

// Re-exports
pub use opcode::Opcode;
pub use format::{Instruction, BytecodeReader, BytecodeWriter};
pub use constants::ConstantPool;
