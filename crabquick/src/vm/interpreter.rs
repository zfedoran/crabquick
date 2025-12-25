//! Bytecode interpreter
//!
//! This module implements the main bytecode execution loop and opcode handlers.

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use crate::bytecode::{BytecodeReader, Opcode, Operand};
use crate::context::Context;
use crate::memory::HeapIndex;
use crate::value::JSValue;
use super::stack::{
    ValueStack, CallStack, StackFrame,
    StackOverflow, StackUnderflow, CallStackOverflow,
};

/// Virtual machine state
pub struct VM {
    /// Value stack for operand evaluation
    value_stack: ValueStack,
    /// Call stack for function frames
    call_stack: CallStack,
    /// Current exception value (if any)
    exception: Option<JSValue>,
    /// Constant pool for current function
    constants: Vec<JSValue>,
    /// Tracks which constants are f64 bits (true) vs JSValue (false)
    const_is_f64: Vec<bool>,
    /// Atom table (index -> string)
    atom_table: Vec<String>,
}

/// VM execution result
pub type VMResult = Result<JSValue, JSValue>;

/// VM error types
#[derive(Debug)]
pub enum VMError {
    StackOverflow,
    StackUnderflow,
    CallStackOverflow,
    InvalidOpcode(u8),
    InvalidOperand,
    TypeError(&'static str),
    RangeError(&'static str),
    ReferenceError(&'static str),
    InternalError(&'static str),
}

impl VM {
    /// Creates a new VM with default stack sizes
    pub fn new() -> Self {
        Self::with_stack_sizes(1000, 100)
    }

    /// Creates a new VM with specified stack sizes
    pub fn with_stack_sizes(value_stack_size: usize, call_stack_depth: usize) -> Self {
        VM {
            value_stack: ValueStack::new(value_stack_size),
            call_stack: CallStack::new(call_stack_depth),
            exception: None,
            constants: Vec::new(),
            const_is_f64: Vec::new(),
            atom_table: Vec::new(),
        }
    }

    /// Executes bytecode in the given context
    ///
    /// Returns the result value or an exception.
    pub fn execute(
        &mut self,
        ctx: &mut Context,
        bytecode_index: HeapIndex,
    ) -> VMResult {
        // Get the bytecode array - check first without holding borrow during error
        let bytecode_ptr: *const crate::value::JSByteArray = match ctx.get_byte_array(bytecode_index) {
            Some(b) => b as *const _,
            None => return Err(self.throw_error(ctx, "Invalid bytecode")),
        };

        // SAFETY: bytecode_ptr is valid as long as we don't modify the arena
        let bytecode_slice = unsafe { (*bytecode_ptr).as_slice() };

        // Parse constant pool and atom table from bytecode
        // Format: [constant_count: u16][(type: u8, value: usize)...]
        //         [atom_count: u16][(len: u16, string_bytes)...]
        //         [bytecode...]
        // Type: 0 = f64 bits, 1 = JSValue
        if bytecode_slice.len() < 2 {
            return Err(self.throw_error(ctx, "Invalid bytecode format"));
        }

        let mut offset = 0;

        // Read constant count
        let const_count = u16::from_le_bytes([bytecode_slice[offset], bytecode_slice[offset + 1]]) as usize;
        offset += 2;

        let const_entry_size = 1 + core::mem::size_of::<usize>(); // type byte + usize value

        if bytecode_slice.len() < offset + const_count * const_entry_size {
            return Err(self.throw_error(ctx, "Invalid bytecode: truncated constant pool"));
        }

        // Read constants
        self.constants.clear();
        self.constants.reserve(const_count);
        self.const_is_f64.clear();
        self.const_is_f64.reserve(const_count);

        for i in 0..const_count {
            let const_type = bytecode_slice[offset];
            offset += 1;

            let mut bytes = [0u8; core::mem::size_of::<usize>()];
            bytes.copy_from_slice(&bytecode_slice[offset..offset + core::mem::size_of::<usize>()]);
            offset += core::mem::size_of::<usize>();

            let raw = usize::from_le_bytes(bytes);
            let value = unsafe { core::mem::transmute::<usize, JSValue>(raw) };
            self.constants.push(value);
            self.const_is_f64.push(const_type == 0);
        }

        // Read atom count
        if bytecode_slice.len() < offset + 2 {
            return Err(self.throw_error(ctx, "Invalid bytecode: missing atom count"));
        }
        let atom_count = u16::from_le_bytes([bytecode_slice[offset], bytecode_slice[offset + 1]]) as usize;
        offset += 2;

        // Read atom strings
        self.atom_table.clear();
        self.atom_table.reserve(atom_count);

        for _ in 0..atom_count {
            if bytecode_slice.len() < offset + 2 {
                return Err(self.throw_error(ctx, "Invalid bytecode: truncated atom table"));
            }
            let len = u16::from_le_bytes([bytecode_slice[offset], bytecode_slice[offset + 1]]) as usize;
            offset += 2;

            if bytecode_slice.len() < offset + len {
                return Err(self.throw_error(ctx, "Invalid bytecode: truncated atom string"));
            }
            let string_bytes = &bytecode_slice[offset..offset + len];
            offset += len;

            let string = core::str::from_utf8(string_bytes)
                .map_err(|_| self.throw_error(ctx, "Invalid UTF-8 in atom table"))?;
            self.atom_table.push(string.to_string());
        }

        // Get the actual bytecode after the constant pool and atom table
        let code_slice = &bytecode_slice[offset..];

        // Create a bytecode reader
        let mut reader = BytecodeReader::new(code_slice);

        // Create initial stack frame
        let frame = StackFrame::new(
            JSValue::undefined(), // func
            0,                     // sp
            0,                     // argc
            JSValue::undefined(),  // this
        );

        if self.call_stack.push(frame).is_err() {
            return Err(self.throw_error(ctx, "Call stack overflow"));
        }

        // Main execution loop
        let result = self.run_loop(ctx, &mut reader);

        // Pop the frame
        let _ = self.call_stack.pop();

        result
    }

    /// Main execution loop
    fn run_loop(
        &mut self,
        ctx: &mut Context,
        reader: &mut BytecodeReader,
    ) -> VMResult {
        loop {
            // Check if we have a pending exception
            if let Some(exc) = self.exception.take() {
                return Err(exc);
            }

            // Decode next instruction
            let pc = reader.pc();
            let instruction = match reader.decode() {
                Some(inst) => inst,
                None => {
                    // End of bytecode - return undefined
                    return Ok(JSValue::undefined());
                }
            };

            // Update PC in current frame
            if let Ok(frame) = self.call_stack.current_mut() {
                frame.pc = pc;
            }

            // Execute the instruction
            match self.execute_instruction(ctx, reader, &instruction) {
                Ok(Some(ret)) => return Ok(ret), // Return instruction
                Ok(None) => continue,              // Normal continuation
                Err(e) => {
                    // Check if we have an exception handler
                    if let Ok(frame) = self.call_stack.current() {
                        if let Some(catch_pc) = frame.catch_offset {
                            // Jump to exception handler
                            reader.set_pc(catch_pc);
                            self.value_stack.push(e)
                                .map_err(|_| self.throw_error(ctx, "Stack overflow in exception handler"))?;
                            continue;
                        }
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Executes a single instruction
    ///
    /// Returns:
    /// - Ok(Some(value)) if this is a return instruction
    /// - Ok(None) for normal instructions
    /// - Err(value) on exception
    fn execute_instruction(
        &mut self,
        ctx: &mut Context,
        reader: &mut BytecodeReader,
        instruction: &crate::bytecode::Instruction,
    ) -> Result<Option<JSValue>, JSValue> {
        use Opcode::*;

        match instruction.opcode {
            // ===== Stack Manipulation =====
            Drop => {
                self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                Ok(None)
            }

            Dup => {
                self.value_stack.dup()
                    .map_err(|_| self.throw_error(ctx, "Stack error"))?;
                Ok(None)
            }

            Swap => {
                self.value_stack.swap()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                Ok(None)
            }

            Nip => {
                // Remove second value (keep top): [a, b] -> [b]
                let top = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                self.value_stack.push(top)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Insert2 => {
                // [a, b, c] -> [c, a, b]
                let c = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                self.value_stack.push(c).ok();
                self.value_stack.push(a).ok();
                self.value_stack.push(b)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Insert3 => {
                // [a, b, c, d] -> [d, a, b, c]
                let d = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let c = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                self.value_stack.push(d).ok();
                self.value_stack.push(a).ok();
                self.value_stack.push(b).ok();
                self.value_stack.push(c)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Perm3 => {
                // [a, b, c] -> [c, b, a]
                self.value_stack.rotate(3, false)
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                Ok(None)
            }

            Rot3l => {
                // [a, b, c] -> [b, c, a]
                self.value_stack.rotate(3, true)
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                Ok(None)
            }

            Rot3r => {
                // [a, b, c] -> [c, a, b]
                self.value_stack.rotate(3, false)
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                Ok(None)
            }

            Rot4l => {
                // [a, b, c, d] -> [b, c, d, a]
                self.value_stack.rotate(4, true)
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                Ok(None)
            }

            // ===== Push Operations =====
            Undefined => {
                self.value_stack.push(JSValue::undefined())
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Null => {
                self.value_stack.push(JSValue::null())
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PushFalse => {
                self.value_stack.push(JSValue::bool(false))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PushTrue => {
                self.value_stack.push(JSValue::bool(true))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PushI8 => {
                if let Operand::I8(val) = instruction.operand {
                    self.value_stack.push(JSValue::from_int(val as i32))
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            PushI16 => {
                if let Operand::I16(val) = instruction.operand {
                    self.value_stack.push(JSValue::from_int(val as i32))
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            PushI32 => {
                if let Operand::I32(val) = instruction.operand {
                    self.value_stack.push(JSValue::from_int(val))
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            PushConst8 => {
                if let Operand::Const8(idx) = instruction.operand {
                    let value = self.get_constant(ctx, idx as u16)?;
                    self.value_stack.push(value)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PushConst8"))
                }
            }

            PushConst16 => {
                if let Operand::Const16(idx) = instruction.operand {
                    let value = self.get_constant(ctx, idx)?;
                    self.value_stack.push(value)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PushConst16"))
                }
            }

            PushMinus1 => {
                self.value_stack.push(JSValue::from_int(-1))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Push0 => {
                self.value_stack.push(JSValue::from_int(0))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Push1 => {
                self.value_stack.push(JSValue::from_int(1))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Push2 => {
                self.value_stack.push(JSValue::from_int(2))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Push3 => {
                self.value_stack.push(JSValue::from_int(3))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Push4 => {
                self.value_stack.push(JSValue::from_int(4))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Push5 => {
                self.value_stack.push(JSValue::from_int(5))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Push6 => {
                self.value_stack.push(JSValue::from_int(6))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Push7 => {
                self.value_stack.push(JSValue::from_int(7))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PushEmptyString => {
                let val = ctx.new_string("")
                    .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                self.value_stack.push(val)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PushThis => {
                let this_val = self.call_stack.current()
                    .map(|f| f.this)
                    .unwrap_or(JSValue::undefined());
                self.value_stack.push(this_val)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PushNaN => {
                let val = ctx.new_number(f64::NAN)
                    .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                self.value_stack.push(val)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PushInfinity => {
                let val = ctx.new_number(f64::INFINITY)
                    .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                self.value_stack.push(val)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PushNegInfinity => {
                let val = ctx.new_number(f64::NEG_INFINITY)
                    .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                self.value_stack.push(val)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            // ===== Arithmetic Operations =====
            Add => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_add(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Sub => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_sub(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Mul => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_mul(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Div => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_div(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Mod => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_mod(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Pow => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_pow(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Plus => {
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.to_number(ctx, a)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Neg => {
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_neg(ctx, a)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Inc => {
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_inc(ctx, a)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Dec => {
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_dec(ctx, a)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PostInc => {
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                // Post-increment returns original value, then increments
                let num = self.to_number(ctx, a)?;
                self.value_stack.push(num)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PostDec => {
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                // Post-decrement returns original value, then decrements
                let num = self.to_number(ctx, a)?;
                self.value_stack.push(num)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            // ===== Comparison Operations =====
            Lt => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_lt(ctx, a, b)?;
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Lte => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_lte(ctx, a, b)?;
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Gt => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_gt(ctx, a, b)?;
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Gte => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_gte(ctx, a, b)?;
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Eq => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_eq(ctx, a, b);
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Neq => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = !self.op_eq(ctx, a, b);
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            StrictEq => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_strict_eq(a, b);
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            StrictNeq => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = !self.op_strict_eq(a, b);
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            // ===== Logical Operations =====
            LNot => {
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = !self.to_boolean(a);
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            LAnd => {
                // Logical AND with short-circuit
                if let Operand::Label(offset) = instruction.operand {
                    let a = self.value_stack.peek()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    if !self.to_boolean(a) {
                        // Short-circuit: jump and keep 'a' on stack
                        reader.set_pc((reader.pc() as i32 + offset) as usize);
                    } else {
                        // Continue: pop 'a', evaluate 'b'
                        self.value_stack.pop().ok();
                    }
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            LOr => {
                // Logical OR with short-circuit
                if let Operand::Label(offset) = instruction.operand {
                    let a = self.value_stack.peek()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    if self.to_boolean(a) {
                        // Short-circuit: jump and keep 'a' on stack
                        reader.set_pc((reader.pc() as i32 + offset) as usize);
                    } else {
                        // Continue: pop 'a', evaluate 'b'
                        self.value_stack.pop().ok();
                    }
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            Nullish => {
                // Nullish coalescing (??)
                if let Operand::Label(offset) = instruction.operand {
                    let a = self.value_stack.peek()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    if !a.is_null() && !a.is_undefined() {
                        // Not nullish: jump and keep 'a' on stack
                        reader.set_pc((reader.pc() as i32 + offset) as usize);
                    } else {
                        // Nullish: pop 'a', evaluate 'b'
                        self.value_stack.pop().ok();
                    }
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            // ===== Bitwise Operations =====
            Not => {
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_bit_not(ctx, a)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            And => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_bit_and(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Or => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_bit_or(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Xor => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_bit_xor(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Shl => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_shl(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Sar => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_sar(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Shr => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = self.op_shr(ctx, a, b)?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            // ===== Control Flow =====
            IfFalse => {
                if let Operand::Label(offset) = instruction.operand {
                    let cond = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    if !self.to_boolean(cond) {
                        reader.set_pc((reader.pc() as i32 + offset) as usize);
                    }
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            IfTrue => {
                if let Operand::Label(offset) = instruction.operand {
                    let cond = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    if self.to_boolean(cond) {
                        reader.set_pc((reader.pc() as i32 + offset) as usize);
                    }
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            Goto => {
                if let Operand::Label(offset) = instruction.operand {
                    reader.set_pc((reader.pc() as i32 + offset) as usize);
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            Return => {
                let ret_val = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                Ok(Some(ret_val))
            }

            ReturnUndef => {
                Ok(Some(JSValue::undefined()))
            }

            // ===== Exception Handling =====
            Throw => {
                let exc = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                Err(exc)
            }

            Catch => {
                // Push the current exception onto the stack
                if let Some(exc) = self.exception.take() {
                    self.value_stack.push(exc)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                } else {
                    self.value_stack.push(JSValue::undefined())
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                }
                Ok(None)
            }

            PushCatchOffset => {
                if let Operand::Label(offset) = instruction.operand {
                    let catch_pc = (reader.pc() as i32 + offset) as usize;
                    if let Ok(frame) = self.call_stack.current_mut() {
                        frame.set_catch_offset(catch_pc);
                    }
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            Rethrow => {
                let exc = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                Err(exc)
            }

            // ===== Object Operations =====
            Object => {
                if let Operand::U8(_count) = instruction.operand {
                    // Create a new object
                    let obj = ctx.new_object()
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                    self.value_stack.push(obj)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            Array => {
                if let Operand::U8(_count) = instruction.operand {
                    // Create a new array
                    let arr = ctx.new_object()
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                    self.value_stack.push(arr)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand"))
                }
            }

            TypeOf => {
                let val = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let type_str = self.typeof_value(val);
                let result = ctx.new_string(type_str)
                    .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Void => {
                // Pop value and push undefined
                self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                self.value_stack.push(JSValue::undefined())
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            // Nop - no operation
            Nop => Ok(None),

            // ===== Global Variable Access =====
            GetGlobal8 => {
                if let Operand::Atom8(atom_idx) = instruction.operand {
                    let atom = self.get_atom_from_table(atom_idx as usize)?;
                    let value = ctx.get_global_property(atom)
                        .unwrap_or(JSValue::undefined());
                    self.value_stack.push(value)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for GetGlobal8"))
                }
            }

            GetGlobal16 => {
                if let Operand::Atom16(atom_idx) = instruction.operand {
                    let atom = self.get_atom_from_table(atom_idx as usize)?;
                    let value = ctx.get_global_property(atom)
                        .unwrap_or(JSValue::undefined());
                    self.value_stack.push(value)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for GetGlobal16"))
                }
            }

            PutGlobal8 => {
                if let Operand::Atom8(atom_idx) = instruction.operand {
                    let atom = self.get_atom_from_table(atom_idx as usize)?;
                    let value = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    ctx.set_global_property(atom, value)
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PutGlobal8"))
                }
            }

            PutGlobal16 => {
                if let Operand::Atom16(atom_idx) = instruction.operand {
                    let atom = self.get_atom_from_table(atom_idx as usize)?;
                    let value = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    ctx.set_global_property(atom, value)
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PutGlobal16"))
                }
            }

            SetGlobal8 => {
                if let Operand::Atom8(atom_idx) = instruction.operand {
                    let atom = self.get_atom_from_table(atom_idx as usize)?;
                    let value = self.value_stack.peek()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    ctx.set_global_property(atom, value)
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for SetGlobal8"))
                }
            }

            SetGlobal16 => {
                if let Operand::Atom16(atom_idx) = instruction.operand {
                    let atom = self.get_atom_from_table(atom_idx as usize)?;
                    let value = self.value_stack.peek()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    ctx.set_global_property(atom, value)
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for SetGlobal16"))
                }
            }

            // ===== Function Calls =====
            Call => {
                if let Operand::U8(argc) = instruction.operand {
                    let argc = argc as u16;
                    // Stack layout: [func, arg1, arg2, ..., argN]
                    // Pop arguments
                    let mut args = alloc::vec::Vec::new();
                    for _ in 0..argc {
                        let arg = self.value_stack.pop()
                            .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                        args.push(arg);
                    }
                    // Reverse args since we popped them in reverse order
                    args.reverse();

                    // Pop function
                    let func = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                    // Call the function
                    let result = ctx.call_function(func, JSValue::undefined(), &args)?;

                    // Push result
                    self.value_stack.push(result)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for Call"))
                }
            }

            CallMethod => {
                if let Operand::U8(argc) = instruction.operand {
                    let argc = argc as u16;
                    // Stack layout: [obj, func, arg1, arg2, ..., argN]
                    // Pop arguments
                    let mut args = alloc::vec::Vec::new();
                    for _ in 0..argc {
                        let arg = self.value_stack.pop()
                            .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                        args.push(arg);
                    }
                    // Reverse args since we popped them in reverse order
                    args.reverse();

                    // Pop function
                    let func = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                    // Pop object (this)
                    let this = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                    // Call the function with 'this'
                    let result = ctx.call_function(func, this, &args)?;

                    // Push result
                    self.value_stack.push(result)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for CallMethod"))
                }
            }

            // ===== Property Access =====
            GetField => {
                if let Operand::U16(atom_idx) = instruction.operand {
                    // Pop object from stack
                    let obj = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                    // Get property atom
                    let atom = self.get_atom_from_table(atom_idx as usize)?;

                    // Get property value
                    let value = ctx.get_property(obj, atom)
                        .unwrap_or(JSValue::undefined());

                    // Push result
                    self.value_stack.push(value)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for GetField"))
                }
            }

            GetField8 => {
                if let Operand::Atom8(atom_idx) = instruction.operand {
                    // Pop object from stack
                    let obj = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                    // Get property atom
                    let atom = self.get_atom_from_table(atom_idx as usize)?;

                    // Get property value
                    let value = ctx.get_property(obj, atom)
                        .unwrap_or(JSValue::undefined());

                    // Push result
                    self.value_stack.push(value)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for GetField8"))
                }
            }

            // ===== Unimplemented Opcodes =====
            // These are stubs that need full implementation
            _ => {
                // For now, return undefined for unimplemented opcodes
                // In a full implementation, each would be properly handled
                Ok(None)
            }
        }
    }

    /// Helper: Throws an error with the given message
    fn throw_error(&mut self, ctx: &mut Context, msg: &str) -> JSValue {
        // Create error string
        let err_msg = ctx.new_string(msg).unwrap_or(JSValue::undefined());
        err_msg
    }

    /// Helper: Gets a constant from the constant pool
    /// For f64 constants, creates a new heap-allocated number
    fn get_constant(&self, ctx: &mut Context, idx: u16) -> Result<JSValue, JSValue> {
        if (idx as usize) >= self.constants.len() {
            let err = ctx.new_string("Constant index out of bounds").unwrap_or(JSValue::undefined());
            return Err(err);
        }

        let value = self.constants[idx as usize];
        let is_f64 = self.const_is_f64.get(idx as usize).copied().unwrap_or(false);

        // Check if this is a raw f64 using the type flag
        if is_f64 {
            // It's raw f64 bits - convert to heap number
            let bits = value.as_raw() as u64;
            let f = f64::from_bits(bits);
            let err = ctx.new_string("Out of memory").unwrap_or(JSValue::undefined());
            return ctx.new_number(f).map_err(|_| err);
        }

        // For other values (ints, special values), just return as-is
        Ok(value)
    }

    /// Helper: Gets an atom from the atom table and converts it to a JSAtom
    /// Uses the same hash function as the runtime
    fn get_atom_from_table(&self, idx: usize) -> Result<crate::value::JSAtom, JSValue> {
        if idx >= self.atom_table.len() {
            return Err(JSValue::undefined());
        }

        let name = &self.atom_table[idx];

        // Use the same hash function as runtime/init.rs string_to_atom
        let mut hash: u32 = 5381;
        for byte in name.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
        }

        Ok(crate::value::JSAtom::from_id(hash))
    }

    /// Type conversion and operator implementations will be added below...
    /// These are simplified versions for the initial implementation.

    fn to_number(&self, ctx: &mut Context, val: JSValue) -> Result<JSValue, JSValue> {
        if let Some(i) = val.to_int() {
            return Ok(JSValue::from_int(i));
        }

        if let Some(n) = ctx.get_number(val) {
            return ctx.new_number(n).map_err(|_| JSValue::undefined());
        }

        // For other types, return 0 for now (simplified)
        Ok(JSValue::from_int(0))
    }

    fn to_boolean(&self, val: JSValue) -> bool {
        if val.is_null() || val.is_undefined() {
            return false;
        }
        if let Some(b) = val.to_bool() {
            return b;
        }
        if let Some(i) = val.to_int() {
            return i != 0;
        }
        true
    }

    fn typeof_value(&self, val: JSValue) -> &'static str {
        if val.is_undefined() {
            "undefined"
        } else if val.is_null() {
            "object" // typeof null === "object" in JavaScript
        } else if val.is_bool() {
            "boolean"
        } else if val.is_int() {
            "number"
        } else if val.is_ptr() {
            "object" // Simplified - would need to check actual type
        } else {
            "undefined"
        }
    }

    // Arithmetic operators (simplified implementations)
    fn op_add(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        ctx.new_number(a_num + b_num).map_err(|_| JSValue::undefined())
    }

    fn op_sub(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        ctx.new_number(a_num - b_num).map_err(|_| JSValue::undefined())
    }

    fn op_mul(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        ctx.new_number(a_num * b_num).map_err(|_| JSValue::undefined())
    }

    fn op_div(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        ctx.new_number(a_num / b_num).map_err(|_| JSValue::undefined())
    }

    fn op_mod(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        ctx.new_number(a_num % b_num).map_err(|_| JSValue::undefined())
    }

    fn op_pow(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        ctx.new_number(libm::pow(a_num, b_num)).map_err(|_| JSValue::undefined())
    }

    fn op_neg(&self, ctx: &mut Context, a: JSValue) -> Result<JSValue, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        ctx.new_number(-a_num).map_err(|_| JSValue::undefined())
    }

    fn op_inc(&self, ctx: &mut Context, a: JSValue) -> Result<JSValue, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        ctx.new_number(a_num + 1.0).map_err(|_| JSValue::undefined())
    }

    fn op_dec(&self, ctx: &mut Context, a: JSValue) -> Result<JSValue, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        ctx.new_number(a_num - 1.0).map_err(|_| JSValue::undefined())
    }

    // Comparison operators
    fn op_lt(&self, ctx: &Context, a: JSValue, b: JSValue) -> Result<bool, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        Ok(a_num < b_num)
    }

    fn op_lte(&self, ctx: &Context, a: JSValue, b: JSValue) -> Result<bool, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        Ok(a_num <= b_num)
    }

    fn op_gt(&self, ctx: &Context, a: JSValue, b: JSValue) -> Result<bool, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        Ok(a_num > b_num)
    }

    fn op_gte(&self, ctx: &Context, a: JSValue, b: JSValue) -> Result<bool, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        Ok(a_num >= b_num)
    }

    fn op_eq(&self, _ctx: &Context, a: JSValue, b: JSValue) -> bool {
        // Simplified equality (would need full type coercion)
        a == b
    }

    fn op_strict_eq(&self, a: JSValue, b: JSValue) -> bool {
        a == b
    }

    // Bitwise operators
    fn op_bit_not(&self, ctx: &mut Context, a: JSValue) -> Result<JSValue, JSValue> {
        let a_int = a.to_int().unwrap_or(0);
        Ok(JSValue::from_int(!a_int))
    }

    fn op_bit_and(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_int = a.to_int().unwrap_or(0);
        let b_int = b.to_int().unwrap_or(0);
        Ok(JSValue::from_int(a_int & b_int))
    }

    fn op_bit_or(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_int = a.to_int().unwrap_or(0);
        let b_int = b.to_int().unwrap_or(0);
        Ok(JSValue::from_int(a_int | b_int))
    }

    fn op_bit_xor(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_int = a.to_int().unwrap_or(0);
        let b_int = b.to_int().unwrap_or(0);
        Ok(JSValue::from_int(a_int ^ b_int))
    }

    fn op_shl(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_int = a.to_int().unwrap_or(0);
        let b_int = (b.to_int().unwrap_or(0) & 0x1F) as u32;
        Ok(JSValue::from_int(a_int << b_int))
    }

    fn op_sar(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_int = a.to_int().unwrap_or(0);
        let b_int = (b.to_int().unwrap_or(0) & 0x1F) as u32;
        Ok(JSValue::from_int(a_int >> b_int))
    }

    fn op_shr(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_int = a.to_int().unwrap_or(0) as u32;
        let b_int = (b.to_int().unwrap_or(0) & 0x1F) as u32;
        Ok(JSValue::from_int((a_int >> b_int) as i32))
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{BytecodeWriter, Instruction};

    #[test]
    fn test_vm_creation() {
        let vm = VM::new();
        assert!(vm.value_stack.is_empty());
        assert!(vm.call_stack.is_empty());
    }

    #[test]
    fn test_simple_arithmetic() {
        let mut vm = VM::new();
        let mut ctx = Context::new(4096);

        // Create bytecode: push 2, push 3, add, return
        let mut writer = BytecodeWriter::new();
        writer.emit(&Instruction::new(Opcode::Push2));
        writer.emit(&Instruction::new(Opcode::Push3));
        writer.emit(&Instruction::new(Opcode::Add));
        writer.emit(&Instruction::new(Opcode::Return));

        let code = writer.finish();

        // Add constant pool header and atom table: [const_count: u16][constants...][atom_count: u16][atoms...][bytecode...]
        let mut bytecode = Vec::new();
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 constants
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 atoms
        bytecode.extend_from_slice(&code);

        let bc_index = ctx.alloc_byte_array(bytecode.len()).unwrap();

        unsafe {
            let bc_array = ctx.get_byte_array_mut(bc_index).unwrap();
            let slice = bc_array.as_full_mut_slice();
            slice[..bytecode.len()].copy_from_slice(&bytecode);
            bc_array.header_mut().set_count(bytecode.len());
        }

        let result = vm.execute(&mut ctx, bc_index).unwrap();
        // Result should be 5 (inlined integer since it fits in i31)
        assert_eq!(result.to_int(), Some(5));
    }

    #[test]
    fn test_stack_operations() {
        let mut vm = VM::new();
        let mut ctx = Context::new(4096);

        // Test: push 1, push 2, swap, pop -> should get 1
        let mut writer = BytecodeWriter::new();
        writer.emit(&Instruction::new(Opcode::Push1));
        writer.emit(&Instruction::new(Opcode::Push2));
        writer.emit(&Instruction::new(Opcode::Swap));
        writer.emit(&Instruction::new(Opcode::Return));

        let code = writer.finish();

        // Add constant pool header and atom table
        let mut bytecode = Vec::new();
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 constants
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 atoms
        bytecode.extend_from_slice(&code);

        let bc_index = ctx.alloc_byte_array(bytecode.len()).unwrap();

        unsafe {
            let bc_array = ctx.get_byte_array_mut(bc_index).unwrap();
            let slice = bc_array.as_full_mut_slice();
            slice[..bytecode.len()].copy_from_slice(&bytecode);
            bc_array.header_mut().set_count(bytecode.len());
        }

        let result = vm.execute(&mut ctx, bc_index).unwrap();
        assert_eq!(result.to_int(), Some(1));
    }

    #[test]
    fn test_conditional_jump() {
        let mut vm = VM::new();
        let mut ctx = Context::new(4096);

        // Bytecode layout (labels are i32, so 4 bytes each):
        // offset 0: PushFalse (1 byte)
        // offset 1: IfFalse (1 opcode + 4 bytes label = 5 bytes), PC after = 6
        // offset 6: Push1 (1 byte)
        // offset 7: Goto (1 opcode + 4 bytes = 5 bytes), PC after = 12
        // offset 12: Push2 (1 byte)
        // offset 13: Return (1 byte)
        //
        // Test: push false, if_false to Push2, push 1, goto to Return, push 2, return
        // When false: skip Push1+Goto, execute Push2 -> return 2
        let mut writer = BytecodeWriter::new();
        writer.emit(&Instruction::new(Opcode::PushFalse));
        writer.emit(&Instruction::with_label(Opcode::IfFalse, 6)); // Jump from PC=6 to PC=12 (Push2)
        writer.emit(&Instruction::new(Opcode::Push1));
        writer.emit(&Instruction::with_label(Opcode::Goto, 1)); // Jump from PC=12 to PC=13 (Return)
        writer.emit(&Instruction::new(Opcode::Push2));
        writer.emit(&Instruction::new(Opcode::Return));

        let code = writer.finish();

        // Add constant pool header and atom table
        let mut bytecode = Vec::new();
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 constants
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 atoms
        bytecode.extend_from_slice(&code);

        let bc_index = ctx.alloc_byte_array(bytecode.len()).unwrap();

        unsafe {
            let bc_array = ctx.get_byte_array_mut(bc_index).unwrap();
            let slice = bc_array.as_full_mut_slice();
            slice[..bytecode.len()].copy_from_slice(&bytecode);
            bc_array.header_mut().set_count(bytecode.len());
        }

        let result = vm.execute(&mut ctx, bc_index).unwrap();
        assert_eq!(result.to_int(), Some(2));
    }
}
