//! Instruction encoding and decoding

use super::opcode::Opcode;

/// Decoded instruction
pub struct Instruction {
    /// The opcode
    pub opcode: Opcode,
    /// Size of the instruction in bytes
    pub size: usize,
    // TODO: Add operand fields
}

/// Bytecode reader for decoding instructions
pub struct BytecodeReader<'a> {
    // TODO: Implement fields:
    // - bytecode: &'a [u8]
    // - pc: usize
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> BytecodeReader<'a> {
    /// Creates a new bytecode reader
    pub fn new(_bytecode: &'a [u8]) -> Self {
        BytecodeReader {
            _marker: core::marker::PhantomData,
        }
    }

    /// Decodes the next instruction
    pub fn decode(&mut self) -> Option<Instruction> {
        // TODO: Read opcode byte
        // TODO: Decode operands based on opcode
        // TODO: Advance PC
        None
    }
}

/// Bytecode writer for encoding instructions
pub struct BytecodeWriter {
    // TODO: Implement fields:
    // - bytecode: Vec<u8>
    _placeholder: u8,
}

impl BytecodeWriter {
    /// Creates a new bytecode writer
    pub fn new() -> Self {
        BytecodeWriter {
            _placeholder: 0,
        }
    }

    /// Emits an opcode
    pub fn emit_op(&mut self, _opcode: Opcode) {
        // TODO: Write opcode byte
    }

    /// Emits a u8 operand
    pub fn emit_u8(&mut self, _val: u8) {
        // TODO: Write byte
    }

    /// Emits a u16 operand
    pub fn emit_u16(&mut self, _val: u16) {
        // TODO: Write 2 bytes (little-endian)
    }

    /// Emits a u32 operand
    pub fn emit_u32(&mut self, _val: u32) {
        // TODO: Write 4 bytes (little-endian)
    }

    /// Returns the bytecode
    pub fn finish(self) -> alloc::vec::Vec<u8> {
        // TODO: Return bytecode
        alloc::vec::Vec::new()
    }
}

impl Default for BytecodeWriter {
    fn default() -> Self {
        Self::new()
    }
}
