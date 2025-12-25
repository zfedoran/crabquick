//! Instruction encoding and decoding
//!
//! This module provides facilities for encoding and decoding bytecode instructions.
//! Instructions consist of an opcode byte followed by 0-4 operand bytes depending
//! on the instruction format.

use super::opcode::{Opcode, InstructionFormat};
use alloc::vec::Vec;

/// Decoded instruction with operands
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    /// The opcode
    pub opcode: Opcode,
    /// The operand (if any)
    pub operand: Operand,
}

/// Instruction operand
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    /// No operand
    None,
    /// Unsigned 8-bit integer
    U8(u8),
    /// Signed 8-bit integer
    I8(i8),
    /// Unsigned 16-bit integer
    U16(u16),
    /// Signed 16-bit integer
    I16(i16),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Signed 32-bit integer
    I32(i32),
    /// Jump label (PC offset)
    Label(i32),
    /// Constant pool index (8-bit)
    Const8(u8),
    /// Constant pool index (16-bit)
    Const16(u16),
    /// Atom index (8-bit)
    Atom8(u8),
    /// Atom index (16-bit)
    Atom16(u16),
}

impl Instruction {
    /// Returns the size of this instruction in bytes
    pub fn size(&self) -> usize {
        self.opcode.size()
    }

    /// Creates a new instruction with no operand
    pub fn new(opcode: Opcode) -> Self {
        Instruction {
            opcode,
            operand: Operand::None,
        }
    }

    /// Creates a new instruction with a u8 operand
    pub fn with_u8(opcode: Opcode, val: u8) -> Self {
        Instruction {
            opcode,
            operand: Operand::U8(val),
        }
    }

    /// Creates a new instruction with an i8 operand
    pub fn with_i8(opcode: Opcode, val: i8) -> Self {
        Instruction {
            opcode,
            operand: Operand::I8(val),
        }
    }

    /// Creates a new instruction with a u16 operand
    pub fn with_u16(opcode: Opcode, val: u16) -> Self {
        Instruction {
            opcode,
            operand: Operand::U16(val),
        }
    }

    /// Creates a new instruction with an i16 operand
    pub fn with_i16(opcode: Opcode, val: i16) -> Self {
        Instruction {
            opcode,
            operand: Operand::I16(val),
        }
    }

    /// Creates a new instruction with a u32 operand
    pub fn with_u32(opcode: Opcode, val: u32) -> Self {
        Instruction {
            opcode,
            operand: Operand::U32(val),
        }
    }

    /// Creates a new instruction with an i32 operand
    pub fn with_i32(opcode: Opcode, val: i32) -> Self {
        Instruction {
            opcode,
            operand: Operand::I32(val),
        }
    }

    /// Creates a new instruction with a label operand
    pub fn with_label(opcode: Opcode, offset: i32) -> Self {
        Instruction {
            opcode,
            operand: Operand::Label(offset),
        }
    }

    /// Creates a new instruction with a const8 operand
    pub fn with_const8(opcode: Opcode, index: u8) -> Self {
        Instruction {
            opcode,
            operand: Operand::Const8(index),
        }
    }

    /// Creates a new instruction with a const16 operand
    pub fn with_const16(opcode: Opcode, index: u16) -> Self {
        Instruction {
            opcode,
            operand: Operand::Const16(index),
        }
    }

    /// Creates a new instruction with an atom8 operand
    pub fn with_atom8(opcode: Opcode, index: u8) -> Self {
        Instruction {
            opcode,
            operand: Operand::Atom8(index),
        }
    }

    /// Creates a new instruction with an atom16 operand
    pub fn with_atom16(opcode: Opcode, index: u16) -> Self {
        Instruction {
            opcode,
            operand: Operand::Atom16(index),
        }
    }
}

/// Bytecode reader for decoding instructions
pub struct BytecodeReader<'a> {
    bytecode: &'a [u8],
    pc: usize,
}

impl<'a> BytecodeReader<'a> {
    /// Creates a new bytecode reader
    pub fn new(bytecode: &'a [u8]) -> Self {
        BytecodeReader { bytecode, pc: 0 }
    }

    /// Returns the current program counter
    pub fn pc(&self) -> usize {
        self.pc
    }

    /// Sets the program counter
    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    /// Returns true if there are more bytes to read
    pub fn has_more(&self) -> bool {
        self.pc < self.bytecode.len()
    }

    /// Reads a u8 from the bytecode
    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pc < self.bytecode.len() {
            let val = self.bytecode[self.pc];
            self.pc += 1;
            Some(val)
        } else {
            None
        }
    }

    /// Reads an i8 from the bytecode
    fn read_i8(&mut self) -> Option<i8> {
        self.read_u8().map(|v| v as i8)
    }

    /// Reads a u16 from the bytecode (little-endian)
    fn read_u16(&mut self) -> Option<u16> {
        if self.pc + 2 <= self.bytecode.len() {
            let val = u16::from_le_bytes([
                self.bytecode[self.pc],
                self.bytecode[self.pc + 1],
            ]);
            self.pc += 2;
            Some(val)
        } else {
            None
        }
    }

    /// Reads an i16 from the bytecode (little-endian)
    fn read_i16(&mut self) -> Option<i16> {
        self.read_u16().map(|v| v as i16)
    }

    /// Reads a u32 from the bytecode (little-endian)
    fn read_u32(&mut self) -> Option<u32> {
        if self.pc + 4 <= self.bytecode.len() {
            let val = u32::from_le_bytes([
                self.bytecode[self.pc],
                self.bytecode[self.pc + 1],
                self.bytecode[self.pc + 2],
                self.bytecode[self.pc + 3],
            ]);
            self.pc += 4;
            Some(val)
        } else {
            None
        }
    }

    /// Reads an i32 from the bytecode (little-endian)
    fn read_i32(&mut self) -> Option<i32> {
        self.read_u32().map(|v| v as i32)
    }

    /// Decodes the next instruction
    pub fn decode(&mut self) -> Option<Instruction> {
        let opcode_byte = self.read_u8()?;
        let opcode = Opcode::from_u8(opcode_byte)?;

        let operand = match opcode.format() {
            InstructionFormat::None => Operand::None,
            InstructionFormat::U8 => Operand::U8(self.read_u8()?),
            InstructionFormat::I8 => Operand::I8(self.read_i8()?),
            InstructionFormat::U16 => Operand::U16(self.read_u16()?),
            InstructionFormat::I16 => Operand::I16(self.read_i16()?),
            InstructionFormat::U32 => Operand::U32(self.read_u32()?),
            InstructionFormat::I32 => Operand::I32(self.read_i32()?),
            InstructionFormat::Label => Operand::Label(self.read_i32()?),
            InstructionFormat::Const8 => Operand::Const8(self.read_u8()?),
            InstructionFormat::Const16 => Operand::Const16(self.read_u16()?),
            InstructionFormat::Atom8 => Operand::Atom8(self.read_u8()?),
            InstructionFormat::Atom16 => Operand::Atom16(self.read_u16()?),
        };

        Some(Instruction { opcode, operand })
    }

    /// Peeks at the next instruction without advancing the PC
    pub fn peek(&mut self) -> Option<Instruction> {
        let saved_pc = self.pc;
        let instruction = self.decode();
        self.pc = saved_pc;
        instruction
    }
}

/// Bytecode writer for encoding instructions
pub struct BytecodeWriter {
    bytecode: Vec<u8>,
}

impl BytecodeWriter {
    /// Creates a new bytecode writer
    pub fn new() -> Self {
        BytecodeWriter {
            bytecode: Vec::new(),
        }
    }

    /// Creates a new bytecode writer with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        BytecodeWriter {
            bytecode: Vec::with_capacity(capacity),
        }
    }

    /// Returns the current bytecode length
    pub fn len(&self) -> usize {
        self.bytecode.len()
    }

    /// Returns true if the bytecode is empty
    pub fn is_empty(&self) -> bool {
        self.bytecode.is_empty()
    }

    /// Returns the current PC (same as len)
    pub fn pc(&self) -> usize {
        self.bytecode.len()
    }

    /// Emits an opcode
    pub fn emit_op(&mut self, opcode: Opcode) {
        self.bytecode.push(opcode as u8);
    }

    /// Emits a u8 operand
    pub fn emit_u8(&mut self, val: u8) {
        self.bytecode.push(val);
    }

    /// Emits an i8 operand
    pub fn emit_i8(&mut self, val: i8) {
        self.bytecode.push(val as u8);
    }

    /// Emits a u16 operand (little-endian)
    pub fn emit_u16(&mut self, val: u16) {
        self.bytecode.extend_from_slice(&val.to_le_bytes());
    }

    /// Emits an i16 operand (little-endian)
    pub fn emit_i16(&mut self, val: i16) {
        self.bytecode.extend_from_slice(&val.to_le_bytes());
    }

    /// Emits a u32 operand (little-endian)
    pub fn emit_u32(&mut self, val: u32) {
        self.bytecode.extend_from_slice(&val.to_le_bytes());
    }

    /// Emits an i32 operand (little-endian)
    pub fn emit_i32(&mut self, val: i32) {
        self.bytecode.extend_from_slice(&val.to_le_bytes());
    }

    /// Emits a complete instruction
    pub fn emit(&mut self, instruction: &Instruction) {
        self.emit_op(instruction.opcode);

        match instruction.operand {
            Operand::None => {}
            Operand::U8(v) => self.emit_u8(v),
            Operand::I8(v) => self.emit_i8(v),
            Operand::U16(v) => self.emit_u16(v),
            Operand::I16(v) => self.emit_i16(v),
            Operand::U32(v) => self.emit_u32(v),
            Operand::I32(v) => self.emit_i32(v),
            Operand::Label(v) => self.emit_i32(v),
            Operand::Const8(v) => self.emit_u8(v),
            Operand::Const16(v) => self.emit_u16(v),
            Operand::Atom8(v) => self.emit_u8(v),
            Operand::Atom16(v) => self.emit_u16(v),
        }
    }

    /// Patches a u32 at the specified offset
    pub fn patch_u32(&mut self, offset: usize, val: u32) {
        if offset + 4 <= self.bytecode.len() {
            let bytes = val.to_le_bytes();
            self.bytecode[offset..offset + 4].copy_from_slice(&bytes);
        }
    }

    /// Patches an i32 at the specified offset
    pub fn patch_i32(&mut self, offset: usize, val: i32) {
        self.patch_u32(offset, val as u32);
    }

    /// Returns a reference to the bytecode
    pub fn as_slice(&self) -> &[u8] {
        &self.bytecode
    }

    /// Returns the bytecode
    pub fn finish(self) -> Vec<u8> {
        self.bytecode
    }

    /// Returns a mutable reference to the bytecode
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytecode
    }
}

impl Default for BytecodeWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_creation() {
        let inst = Instruction::new(Opcode::Drop);
        assert_eq!(inst.opcode, Opcode::Drop);
        assert_eq!(inst.operand, Operand::None);
        assert_eq!(inst.size(), 1);

        let inst = Instruction::with_i8(Opcode::PushI8, 42);
        assert_eq!(inst.opcode, Opcode::PushI8);
        assert_eq!(inst.operand, Operand::I8(42));
        assert_eq!(inst.size(), 2);

        let inst = Instruction::with_label(Opcode::IfFalse, 100);
        assert_eq!(inst.opcode, Opcode::IfFalse);
        assert_eq!(inst.operand, Operand::Label(100));
        assert_eq!(inst.size(), 5);
    }

    #[test]
    fn test_writer_basic() {
        let mut writer = BytecodeWriter::new();

        // Emit drop instruction
        writer.emit_op(Opcode::Drop);
        assert_eq!(writer.len(), 1);
        assert_eq!(writer.as_slice(), &[0]);

        // Emit push_i8 42
        writer.emit_op(Opcode::PushI8);
        writer.emit_i8(42);
        assert_eq!(writer.len(), 3);
        assert_eq!(writer.as_slice(), &[0, 14, 42]);

        // Emit add
        writer.emit_op(Opcode::Add);
        assert_eq!(writer.len(), 4);
        assert_eq!(writer.as_slice(), &[0, 14, 42, 90]);
    }

    #[test]
    fn test_writer_emit_instruction() {
        let mut writer = BytecodeWriter::new();

        writer.emit(&Instruction::new(Opcode::Drop));
        writer.emit(&Instruction::with_i8(Opcode::PushI8, 42));
        writer.emit(&Instruction::new(Opcode::Add));

        assert_eq!(writer.as_slice(), &[0, 14, 42, 90]);
    }

    #[test]
    fn test_writer_u16() {
        let mut writer = BytecodeWriter::new();
        writer.emit_u16(0x1234);
        assert_eq!(writer.as_slice(), &[0x34, 0x12]); // little-endian
    }

    #[test]
    fn test_writer_u32() {
        let mut writer = BytecodeWriter::new();
        writer.emit_u32(0x12345678);
        assert_eq!(writer.as_slice(), &[0x78, 0x56, 0x34, 0x12]); // little-endian
    }

    #[test]
    fn test_writer_patch() {
        let mut writer = BytecodeWriter::new();
        writer.emit_op(Opcode::IfFalse);
        let patch_offset = writer.pc();
        writer.emit_i32(0); // Placeholder
        writer.emit_op(Opcode::Return);

        // Patch the jump offset
        writer.patch_i32(patch_offset, 100);

        let expected = &[160, 100, 0, 0, 0, 163];
        assert_eq!(writer.as_slice(), expected);
    }

    #[test]
    fn test_reader_basic() {
        let bytecode = vec![0, 14, 42, 90]; // drop, push_i8 42, add
        let mut reader = BytecodeReader::new(&bytecode);

        assert_eq!(reader.pc(), 0);
        assert!(reader.has_more());

        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::Drop);
        assert_eq!(inst.operand, Operand::None);
        assert_eq!(reader.pc(), 1);

        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::PushI8);
        assert_eq!(inst.operand, Operand::I8(42));
        assert_eq!(reader.pc(), 3);

        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::Add);
        assert_eq!(inst.operand, Operand::None);
        assert_eq!(reader.pc(), 4);

        assert!(!reader.has_more());
        assert!(reader.decode().is_none());
    }

    #[test]
    fn test_reader_u16() {
        let bytecode = vec![70, 0x34, 0x12]; // get_field 0x1234
        let mut reader = BytecodeReader::new(&bytecode);

        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::GetField);
        assert_eq!(inst.operand, Operand::U16(0x1234));
    }

    #[test]
    fn test_reader_i32() {
        let bytecode = vec![160, 0x78, 0x56, 0x34, 0x12]; // if_false 0x12345678
        let mut reader = BytecodeReader::new(&bytecode);

        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::IfFalse);
        assert_eq!(inst.operand, Operand::Label(0x12345678));
    }

    #[test]
    fn test_reader_peek() {
        let bytecode = vec![0, 90]; // drop, add
        let mut reader = BytecodeReader::new(&bytecode);

        let inst = reader.peek().unwrap();
        assert_eq!(inst.opcode, Opcode::Drop);
        assert_eq!(reader.pc(), 0); // PC unchanged

        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::Drop);
        assert_eq!(reader.pc(), 1);
    }

    #[test]
    fn test_reader_set_pc() {
        let bytecode = vec![0, 90, 163]; // drop, add, return
        let mut reader = BytecodeReader::new(&bytecode);

        reader.set_pc(2);
        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::Return);
    }

    #[test]
    fn test_roundtrip() {
        let mut writer = BytecodeWriter::new();
        writer.emit(&Instruction::new(Opcode::Drop));
        writer.emit(&Instruction::with_i8(Opcode::PushI8, 42));
        writer.emit(&Instruction::with_i8(Opcode::PushI8, 10));
        writer.emit(&Instruction::new(Opcode::Add));
        writer.emit(&Instruction::new(Opcode::Return));

        let bytecode = writer.finish();
        let mut reader = BytecodeReader::new(&bytecode);

        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::Drop);

        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::PushI8);
        assert_eq!(inst.operand, Operand::I8(42));

        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::PushI8);
        assert_eq!(inst.operand, Operand::I8(10));

        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::Add);

        let inst = reader.decode().unwrap();
        assert_eq!(inst.opcode, Opcode::Return);

        assert!(!reader.has_more());
    }
}
