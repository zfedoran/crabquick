//! Bytecode format and instruction encoding
//!
//! Defines opcodes, instruction encoding, constant pool management,
//! and function bytecode structures.

pub mod opcode;
pub mod format;
pub mod constants;
pub mod function;

// Re-exports
pub use opcode::{Opcode, InstructionFormat};
pub use format::{Instruction, Operand, BytecodeReader, BytecodeWriter};
pub use constants::ConstantPool;
pub use function::JSFunctionBytecode;
