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

/// Function entry from function table
#[derive(Debug, Clone)]
struct FunctionEntry {
    bytecode_index: HeapIndex,
    param_count: u8,
    local_count: u8,
}

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
    /// Function table (precompiled functions)
    function_table: Vec<FunctionEntry>,
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
            function_table: Vec::new(),
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

        // Read function table
        if bytecode_slice.len() < offset + 2 {
            return Err(self.throw_error(ctx, "Invalid bytecode: missing function count"));
        }
        let func_count = u16::from_le_bytes([bytecode_slice[offset], bytecode_slice[offset + 1]]) as usize;
        offset += 2;

        self.function_table.clear();
        self.function_table.reserve(func_count);

        for _ in 0..func_count {
            // Read param_count (u8), local_count (u8), bytecode_len (u32), then bytecode bytes
            if bytecode_slice.len() < offset + 6 {
                return Err(self.throw_error(ctx, "Invalid bytecode: truncated function table"));
            }

            let param_count = bytecode_slice[offset];
            offset += 1;
            let local_count = bytecode_slice[offset];
            offset += 1;

            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&bytecode_slice[offset..offset + 4]);
            offset += 4;
            let bytecode_len = u32::from_le_bytes(len_bytes) as usize;

            if bytecode_slice.len() < offset + bytecode_len {
                return Err(self.throw_error(ctx, "Invalid bytecode: truncated function bytecode"));
            }

            let func_bytecode = &bytecode_slice[offset..offset + bytecode_len];
            offset += bytecode_len;

            // Allocate a ByteArray for this function's bytecode
            let func_bc_index = ctx.alloc_byte_array(bytecode_len)
                .map_err(|_| self.throw_error(ctx, "Out of memory allocating function bytecode"))?;

            // Copy the bytecode to the allocated array
            unsafe {
                if let Some(bc_array) = ctx.get_byte_array_mut(func_bc_index) {
                    let slice = bc_array.as_full_mut_slice();
                    slice[..bytecode_len].copy_from_slice(func_bytecode);
                    bc_array.header_mut().set_count(bytecode_len);
                }
            }

            self.function_table.push(FunctionEntry {
                bytecode_index: func_bc_index,
                param_count,
                local_count,
            });
        }

        // Get the actual bytecode after the constant pool, atom table, and function table
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

            PushFunc8 => {
                if let Operand::U8(func_idx) = instruction.operand {
                    // Get function from function table
                    if (func_idx as usize) >= self.function_table.len() {
                        return Err(self.throw_error(ctx, "Function index out of bounds"));
                    }

                    let func_entry = &self.function_table[func_idx as usize];

                    // Create a bytecode function object
                    let func_val = ctx.new_bytecode_function(
                        func_entry.bytecode_index,
                        func_entry.param_count,
                        func_entry.local_count,
                    ).map_err(|_| self.throw_error(ctx, "Out of memory creating function"))?;

                    self.value_stack.push(func_val)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PushFunc8"))
                }
            }

            PushFunc => {
                if let Operand::U16(func_idx) = instruction.operand {
                    // Get function from function table
                    if (func_idx as usize) >= self.function_table.len() {
                        return Err(self.throw_error(ctx, "Function index out of bounds"));
                    }

                    let func_entry = &self.function_table[func_idx as usize];

                    // Create a bytecode function object
                    let func_val = ctx.new_bytecode_function(
                        func_entry.bytecode_index,
                        func_entry.param_count,
                        func_entry.local_count,
                    ).map_err(|_| self.throw_error(ctx, "Out of memory creating function"))?;

                    self.value_stack.push(func_val)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PushFunc"))
                }
            }

            // ===== Closure Operations =====
            FClosure => {
                // FClosure creates a closure object with captured variables
                // The operand is the function index (Const8 format)
                if let Operand::Const8(func_idx) = instruction.operand {
                    // Get function from function table
                    if (func_idx as usize) >= self.function_table.len() {
                        return Err(self.throw_error(ctx, "Function index out of bounds"));
                    }

                    // Get the function entry to extract bytecode_index, param_count, local_count
                    let func_entry = &self.function_table[func_idx as usize];
                    let bytecode_index = func_entry.bytecode_index;
                    let param_count = func_entry.param_count;
                    let local_count = func_entry.local_count;

                    // Get the captured var count from the next byte
                    // The compiler will emit: FClosure func_idx, captured_count, [var_ref indices...]
                    let captured_count = reader.read_u8().unwrap_or(0) as usize;

                    // Collect var ref heap indices
                    let mut var_refs = alloc::vec::Vec::with_capacity(captured_count);

                    for _ in 0..captured_count {
                        // Read the capture source info (local index to capture)
                        let local_idx = reader.read_u8().unwrap_or(0) as usize;

                        // Get the current call frame info (avoiding borrow issues)
                        let (base_sp, parent_closure_opt) = match self.call_stack.current() {
                            Ok(frame) => (frame.sp, frame.closure),
                            Err(_) => return Err(self.throw_error(ctx, "No call frame")),
                        };

                        // Check if we're in a closure context with existing var refs
                        if let Some(parent_closure_idx) = parent_closure_opt {
                            // Get parent closure info
                            let reused_var_ref = match ctx.get_closure(parent_closure_idx) {
                                Some(parent_closure) => {
                                    if local_idx < parent_closure.var_ref_count as usize {
                                        Some(parent_closure.get_var_ref(local_idx))
                                    } else {
                                        None
                                    }
                                }
                                None => return Err(self.throw_error(ctx, "Invalid parent closure")),
                            };

                            if let Some(existing_ref) = reused_var_ref {
                                // Reuse the var ref from parent
                                var_refs.push(existing_ref);
                            } else {
                                // It's a local variable - create new var ref
                                let local_val = self.value_stack.get(base_sp + local_idx)
                                    .unwrap_or(JSValue::undefined());
                                match ctx.alloc_var_ref(local_val) {
                                    Ok(var_ref_idx) => var_refs.push(var_ref_idx),
                                    Err(_) => return Err(self.throw_error(ctx, "Out of memory")),
                                }
                            }
                        } else {
                            // Not in a closure - capture from local stack
                            let local_val = self.value_stack.get(base_sp + local_idx)
                                .unwrap_or(JSValue::undefined());
                            match ctx.alloc_var_ref(local_val) {
                                Ok(var_ref_idx) => var_refs.push(var_ref_idx),
                                Err(_) => return Err(self.throw_error(ctx, "Out of memory")),
                            }
                        }
                    }

                    // Allocate the closure object with bytecode_index (not func table index!)
                    let closure_idx = match ctx.alloc_closure(bytecode_index, param_count, local_count, &var_refs) {
                        Ok(idx) => idx,
                        Err(_) => return Err(self.throw_error(ctx, "Out of memory creating closure")),
                    };

                    // Push closure as a JSValue
                    let closure_val = JSValue::from_ptr(closure_idx);
                    match self.value_stack.push(closure_val) {
                        Ok(()) => Ok(None),
                        Err(_) => Err(self.throw_error(ctx, "Stack overflow")),
                    }
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for FClosure"))
                }
            }

            GetVarRef => {
                // Get a captured variable from the current closure's environment
                if let Operand::U8(var_idx) = instruction.operand {
                    // Get the closure from the current call frame
                    let closure_idx = match self.call_stack.current() {
                        Ok(frame) => match frame.closure {
                            Some(idx) => idx,
                            None => return Err(self.throw_error(ctx, "GetVarRef outside closure")),
                        },
                        Err(_) => return Err(self.throw_error(ctx, "No call frame")),
                    };

                    // Get var ref index from closure
                    let var_ref_idx = match ctx.get_closure(closure_idx) {
                        Some(closure) => {
                            if var_idx >= closure.var_ref_count {
                                return Err(self.throw_error(ctx, "Var ref index out of bounds"));
                            }
                            closure.get_var_ref(var_idx as usize)
                        }
                        None => return Err(self.throw_error(ctx, "Invalid closure")),
                    };

                    // Get the value from var ref
                    let value = match ctx.get_var_ref(var_ref_idx) {
                        Some(var_ref) => var_ref.value(),
                        None => return Err(self.throw_error(ctx, "Invalid var ref")),
                    };

                    match self.value_stack.push(value) {
                        Ok(()) => Ok(None),
                        Err(_) => Err(self.throw_error(ctx, "Stack overflow")),
                    }
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for GetVarRef"))
                }
            }

            PutVarRef => {
                // Set a captured variable (pops value from stack)
                if let Operand::U8(var_idx) = instruction.operand {
                    let value = match self.value_stack.pop() {
                        Ok(v) => v,
                        Err(_) => return Err(self.throw_error(ctx, "Stack underflow")),
                    };

                    // Get the closure from the current call frame
                    let closure_idx = match self.call_stack.current() {
                        Ok(frame) => match frame.closure {
                            Some(idx) => idx,
                            None => return Err(self.throw_error(ctx, "PutVarRef outside closure")),
                        },
                        Err(_) => return Err(self.throw_error(ctx, "No call frame")),
                    };

                    // Get var ref index from closure
                    let var_ref_idx = match ctx.get_closure(closure_idx) {
                        Some(closure) => {
                            if var_idx >= closure.var_ref_count {
                                return Err(self.throw_error(ctx, "Var ref index out of bounds"));
                            }
                            closure.get_var_ref(var_idx as usize)
                        }
                        None => return Err(self.throw_error(ctx, "Invalid closure")),
                    };

                    // Set the value in var ref
                    match ctx.get_var_ref_mut(var_ref_idx) {
                        Some(var_ref) => {
                            var_ref.set_value(value);
                            Ok(None)
                        }
                        None => Err(self.throw_error(ctx, "Invalid var ref")),
                    }
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PutVarRef"))
                }
            }

            SetVarRef => {
                // Set a captured variable (leaves value on stack)
                if let Operand::U8(var_idx) = instruction.operand {
                    let value = match self.value_stack.peek() {
                        Ok(v) => v,
                        Err(_) => return Err(self.throw_error(ctx, "Stack underflow")),
                    };

                    // Get the closure from the current call frame
                    let closure_idx = match self.call_stack.current() {
                        Ok(frame) => match frame.closure {
                            Some(idx) => idx,
                            None => return Err(self.throw_error(ctx, "SetVarRef outside closure")),
                        },
                        Err(_) => return Err(self.throw_error(ctx, "No call frame")),
                    };

                    // Get var ref index from closure
                    let var_ref_idx = match ctx.get_closure(closure_idx) {
                        Some(closure) => {
                            if var_idx >= closure.var_ref_count {
                                return Err(self.throw_error(ctx, "Var ref index out of bounds"));
                            }
                            closure.get_var_ref(var_idx as usize)
                        }
                        None => return Err(self.throw_error(ctx, "Invalid closure")),
                    };

                    // Set the value in var ref
                    match ctx.get_var_ref_mut(var_ref_idx) {
                        Some(var_ref) => {
                            var_ref.set_value(value);
                            Ok(None)
                        }
                        None => Err(self.throw_error(ctx, "Invalid var ref")),
                    }
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for SetVarRef"))
                }
            }

            PushAtomString8 => {
                if let Operand::Atom8(atom_idx) = instruction.operand {
                    // Get string from atom table
                    if (atom_idx as usize) >= self.atom_table.len() {
                        return Err(self.throw_error(ctx, "Atom index out of bounds"));
                    }

                    let string = &self.atom_table[atom_idx as usize];
                    let val = ctx.new_string(string)
                        .map_err(|_| self.throw_error(ctx, "Out of memory creating string"))?;
                    self.value_stack.push(val)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PushAtomString8"))
                }
            }

            PushAtomString16 => {
                if let Operand::Atom16(atom_idx) = instruction.operand {
                    // Get string from atom table
                    if (atom_idx as usize) >= self.atom_table.len() {
                        return Err(self.throw_error(ctx, "Atom index out of bounds"));
                    }

                    let string = &self.atom_table[atom_idx as usize];
                    let val = ctx.new_string(string)
                        .map_err(|_| self.throw_error(ctx, "Out of memory creating string"))?;
                    self.value_stack.push(val)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PushAtomString16"))
                }
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
                let result = self.op_strict_eq(ctx, a, b);
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            StrictNeq => {
                let b = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = !self.op_strict_eq(ctx, a, b);
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            // ===== Logical Operations =====
            LNot => {
                let a = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let result = !self.to_boolean(ctx, a);
                self.value_stack.push(JSValue::bool(result))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            LAnd => {
                // Logical AND with short-circuit
                if let Operand::Label(offset) = instruction.operand {
                    let a = self.value_stack.peek()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    if !self.to_boolean(ctx, a) {
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
                    if self.to_boolean(ctx, a) {
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
                    if !self.to_boolean(ctx, cond) {
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
                    if self.to_boolean(ctx, cond) {
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
                    // Get Array.prototype for proper inheritance
                    let array_atom = crate::runtime::init::string_to_atom("Array");
                    let proto_atom = crate::runtime::init::string_to_atom("prototype");
                    let array_proto = ctx.get_global_property(array_atom)
                        .and_then(|arr_ctor| ctx.get_property(arr_ctor, proto_atom))
                        .unwrap_or(JSValue::null());

                    // Create a new array with Array.prototype
                    let arr = ctx.new_object_with_proto(array_proto)
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;

                    // Initialize length to 0
                    let length_atom = crate::runtime::init::string_to_atom("length");
                    let zero = ctx.new_number(0.0)
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                    ctx.add_property(arr, length_atom, zero, crate::object::PropertyFlags::default())
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
                let type_str = self.typeof_value(ctx, val);
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

                    // Check if it's a closure
                    if ctx.is_closure(func) {
                        // This is a closure - extract the function and closure environment
                        let closure_idx = match func.to_ptr() {
                            Some(idx) => idx,
                            None => return Err(self.throw_error(ctx, "Invalid closure value")),
                        };

                        // Get closure info - now uses bytecode_index directly!
                        let (bytecode_index, param_count, local_count) = match ctx.get_closure(closure_idx) {
                            Some(closure) => (closure.bytecode_index, closure.param_count as usize, closure.local_count as usize),
                            None => return Err(self.throw_error(ctx, "Invalid closure")),
                        };

                        // Pad args if needed (undefined for missing params)
                        while args.len() < param_count {
                            args.push(JSValue::undefined());
                        }

                        // Push arguments onto the stack as locals
                        let base_sp = self.value_stack.len();
                        for arg in &args {
                            self.value_stack.push(*arg)
                                .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        }

                        // Reserve space for additional locals
                        for _ in param_count..local_count {
                            self.value_stack.push(JSValue::undefined())
                                .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        }

                        // Push a call frame to track base_sp for nested closures
                        let frame = StackFrame::new_closure(func, base_sp, args.len() as u16, JSValue::undefined(), closure_idx);
                        self.call_stack.push(frame)
                            .map_err(|_| self.throw_error(ctx, "Call stack overflow"))?;

                        // Execute the function with closure context
                        let result = self.execute_bytecode_function(ctx, bytecode_index, base_sp, local_count, Some(closure_idx));

                        // Pop the call frame
                        let _ = self.call_stack.pop();

                        // Handle any error from execution
                        let result = result?;

                        // Clean up local variables from stack
                        self.value_stack.truncate(base_sp);

                        // Push result
                        self.value_stack.push(result)
                            .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        Ok(None)
                    } else if let Some(bc_func) = ctx.get_bytecode_function(func) {
                        // This is a bytecode function - execute it within the VM
                        // Get the function's bytecode
                        let func_bc_index = bc_func.bytecode_index();
                        let param_count = bc_func.param_count() as usize;
                        let local_count = bc_func.local_count() as usize;

                        // Pad args if needed (undefined for missing params)
                        while args.len() < param_count {
                            args.push(JSValue::undefined());
                        }

                        // Push arguments onto the stack as locals (simplified for now)
                        let base_sp = self.value_stack.len();
                        for arg in &args {
                            self.value_stack.push(*arg)
                                .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        }

                        // Reserve space for additional locals
                        for _ in param_count..local_count {
                            self.value_stack.push(JSValue::undefined())
                                .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        }

                        // Push a call frame to track base_sp for closures
                        let frame = StackFrame::new(func, base_sp, argc, JSValue::undefined());
                        self.call_stack.push(frame)
                            .map_err(|_| self.throw_error(ctx, "Call stack overflow"))?;

                        // Execute the function (no closure context)
                        let result = self.execute_bytecode_function(ctx, func_bc_index, base_sp, local_count, None);

                        // Pop the call frame
                        let _ = self.call_stack.pop();

                        // Handle any error from execution
                        let result = result?;

                        // Clean up local variables from stack
                        self.value_stack.truncate(base_sp);

                        // Push result
                        self.value_stack.push(result)
                            .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        Ok(None)
                    } else {
                        // Not a bytecode function - try native function
                        let result = ctx.call_function(func, JSValue::undefined(), &args)?;

                        // Push result
                        self.value_stack.push(result)
                            .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        Ok(None)
                    }
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
                    let _this = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                    // Handle closures first
                    if ctx.is_closure(func) {
                        let closure_idx = match func.to_ptr() {
                            Some(idx) => idx,
                            None => return Err(self.throw_error(ctx, "Invalid closure value")),
                        };

                        let (bytecode_index, param_count, local_count) = match ctx.get_closure(closure_idx) {
                            Some(closure) => (closure.bytecode_index, closure.param_count as usize, closure.local_count as usize),
                            None => return Err(self.throw_error(ctx, "Invalid closure")),
                        };

                        while args.len() < param_count {
                            args.push(JSValue::undefined());
                        }

                        let base_sp = self.value_stack.len();
                        for arg in &args {
                            self.value_stack.push(*arg)
                                .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        }

                        for _ in param_count..local_count {
                            self.value_stack.push(JSValue::undefined())
                                .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        }

                        let frame = StackFrame::new_closure(func, base_sp, args.len() as u16, JSValue::undefined(), closure_idx);
                        self.call_stack.push(frame)
                            .map_err(|_| self.throw_error(ctx, "Call stack overflow"))?;

                        let result = self.execute_bytecode_function(ctx, bytecode_index, base_sp, local_count, Some(closure_idx));

                        let _ = self.call_stack.pop();
                        let result = result?;

                        self.value_stack.truncate(base_sp);
                        self.value_stack.push(result)
                            .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        Ok(None)
                    } else if let Some(bc_func) = ctx.get_bytecode_function(func) {
                        // Bytecode function
                        let func_bc_index = bc_func.bytecode_index();
                        let param_count = bc_func.param_count() as usize;
                        let local_count = bc_func.local_count() as usize;

                        while args.len() < param_count {
                            args.push(JSValue::undefined());
                        }

                        let base_sp = self.value_stack.len();
                        for arg in &args {
                            self.value_stack.push(*arg)
                                .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        }

                        for _ in param_count..local_count {
                            self.value_stack.push(JSValue::undefined())
                                .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        }

                        let frame = StackFrame::new(func, base_sp, argc, JSValue::undefined());
                        self.call_stack.push(frame)
                            .map_err(|_| self.throw_error(ctx, "Call stack overflow"))?;

                        let result = self.execute_bytecode_function(ctx, func_bc_index, base_sp, local_count, None);

                        let _ = self.call_stack.pop();
                        let result = result?;

                        self.value_stack.truncate(base_sp);
                        self.value_stack.push(result)
                            .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        Ok(None)
                    } else {
                        // Native function - use ctx.call_function
                        let result = ctx.call_function(func, _this, &args)?;
                        self.value_stack.push(result)
                            .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                        Ok(None)
                    }
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

            PutField8 => {
                if let Operand::Atom8(atom_idx) = instruction.operand {
                    // Pop value, then object from stack
                    let value = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    let obj = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                    // Get property atom
                    let atom = self.get_atom_from_table(atom_idx as usize)?;

                    // Set property on object
                    ctx.add_property(obj, atom, value, crate::object::PropertyFlags::default())
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;

                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PutField8"))
                }
            }

            PutField => {
                if let Operand::U16(atom_idx) = instruction.operand {
                    // Pop value, then object from stack
                    let value = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    let obj = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                    // Get property atom
                    let atom = self.get_atom_from_table(atom_idx as usize)?;

                    // Set property on object
                    ctx.add_property(obj, atom, value, crate::object::PropertyFlags::default())
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;

                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PutField"))
                }
            }

            SetField => {
                if let Operand::U16(atom_idx) = instruction.operand {
                    // Pop value, then object from stack
                    let value = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    let obj = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                    // Get property atom
                    let atom = self.get_atom_from_table(atom_idx as usize)?;

                    // Set property on object
                    ctx.add_property(obj, atom, value, crate::object::PropertyFlags::default())
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;

                    // Push value back (SetField returns the assigned value)
                    self.value_stack.push(value)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;

                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for SetField"))
                }
            }

            // ===== Local Variable Access =====
            GetLoc => {
                if let Operand::U8(idx) = instruction.operand {
                    // Get local variable from the value stack
                    // At top level, locals start at the base of the current frame
                    let base_sp = match self.call_stack.current() {
                        Ok(frame) => frame.sp,
                        Err(_) => return Err(self.throw_error(ctx, "No call frame")),
                    };

                    let local_val = self.value_stack.get(base_sp + idx as usize)
                        .map_err(|_| self.throw_error(ctx, "Invalid local variable index"))?;
                    self.value_stack.push(local_val)
                        .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for GetLoc"))
                }
            }

            PutLoc => {
                if let Operand::U8(idx) = instruction.operand {
                    let val = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                    let base_sp = match self.call_stack.current() {
                        Ok(frame) => frame.sp,
                        Err(_) => return Err(self.throw_error(ctx, "No call frame")),
                    };
                    let target_idx = base_sp + idx as usize;

                    // Ensure we have enough space for this local
                    while self.value_stack.len() <= target_idx {
                        self.value_stack.push(JSValue::undefined())
                            .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    }

                    self.value_stack.set(target_idx, val)
                        .map_err(|_| self.throw_error(ctx, "Invalid local variable index"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for PutLoc"))
                }
            }

            SetLoc => {
                if let Operand::U8(idx) = instruction.operand {
                    let val = self.value_stack.peek()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                    let base_sp = match self.call_stack.current() {
                        Ok(frame) => frame.sp,
                        Err(_) => return Err(self.throw_error(ctx, "No call frame")),
                    };
                    let target_idx = base_sp + idx as usize;

                    // Ensure we have enough space for this local
                    while self.value_stack.len() <= target_idx {
                        self.value_stack.push(JSValue::undefined())
                            .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    }

                    self.value_stack.set(target_idx, val)
                        .map_err(|_| self.throw_error(ctx, "Invalid local variable index"))?;
                    Ok(None)
                } else {
                    Err(self.throw_error(ctx, "Invalid operand for SetLoc"))
                }
            }

            // ===== Object/Array Creation =====
            // Note: Array opcode already handled above, this is dead code
            // Keeping for completeness but should be cleaned up

            // ===== Array/Object Element Access =====
            GetArrayEl => {
                // Stack: [obj, index] -> [value]
                let index = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let obj = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                // Convert index to number
                let idx_num = if let Some(i) = index.to_int() {
                    i as f64
                } else if let Some(n) = ctx.get_number(index) {
                    n
                } else {
                    0.0
                };

                // Convert number to property key (toString)
                let key_str = alloc::format!("{}", idx_num as i32);

                // Create atom for the property key
                let mut hash: u32 = 5381;
                for byte in key_str.bytes() {
                    hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
                }
                let key = crate::value::JSAtom::from_id(hash);

                // Get the property
                let value = ctx.get_property(obj, key)
                    .unwrap_or(JSValue::undefined());

                self.value_stack.push(value)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PutArrayEl => {
                // Stack: [obj, index, value] -> [obj]
                let value = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let index = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                let obj = self.value_stack.peek()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                // Convert index to number
                let idx_num = if let Some(i) = index.to_int() {
                    i as f64
                } else if let Some(n) = ctx.get_number(index) {
                    n
                } else {
                    0.0
                };

                // Convert number to property key (toString)
                let key_str = alloc::format!("{}", idx_num as i32);

                // Create atom for the property key
                let mut hash: u32 = 5381;
                for byte in key_str.bytes() {
                    hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
                }
                let key = crate::value::JSAtom::from_id(hash);

                // Set the property
                ctx.add_property(obj, key, value, crate::object::PropertyFlags::default())
                    .map_err(|_| self.throw_error(ctx, "Out of memory"))?;

                // Update length property if this is a numeric index
                if idx_num >= 0.0 && idx_num == libm::floor(idx_num) {
                    let length_atom = crate::runtime::init::string_to_atom("length");

                    // Get current length (defaults to 0)
                    let current_length = ctx.get_property(obj, length_atom)
                        .and_then(|v| ctx.get_number(v))
                        .unwrap_or(0.0);

                    // New length should be max of current and idx + 1
                    let new_length = f64::max(current_length, idx_num + 1.0);

                    let new_length_val = ctx.new_number(new_length)
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;

                    // Always add the property (this creates a duplicate, but get_property
                    // should find the most recent one if the hash table is searched properly)
                    // TODO: Implement proper property update mechanism
                    ctx.add_property(obj, length_atom, new_length_val, crate::object::PropertyFlags::default())
                        .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                }

                // Leave obj on stack
                Ok(None)
            }

            // ===== Increment/Decrement Operators =====
            Inc => {
                // ++x: pop value, increment, push result
                let val = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                let num = if let Some(i) = val.to_int() {
                    i as f64 + 1.0
                } else if let Some(f) = ctx.get_number(val) {
                    f + 1.0
                } else {
                    f64::NAN
                };

                let result = ctx.new_number(num)
                    .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            Dec => {
                // --x: pop value, decrement, push result
                let val = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                let num = if let Some(i) = val.to_int() {
                    i as f64 - 1.0
                } else if let Some(f) = ctx.get_number(val) {
                    f - 1.0
                } else {
                    f64::NAN
                };

                let result = ctx.new_number(num)
                    .map_err(|_| self.throw_error(ctx, "Out of memory"))?;
                self.value_stack.push(result)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            }

            PostInc => {
                // x++: pop value, push original, increment and store
                // Note: This needs special handling in codegen to work with lvalues
                let val = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                let num = if let Some(i) = val.to_int() {
                    i as f64
                } else if let Some(f) = ctx.get_number(val) {
                    f
                } else {
                    f64::NAN
                };

                // Push original value (this is what the expression returns)
                self.value_stack.push(val)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;

                // The incremented value should be stored by the calling code
                // For now, this is a simplified implementation
                Ok(None)
            }

            PostDec => {
                // x--: pop value, push original, decrement and store
                let val = self.value_stack.pop()
                    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                let num = if let Some(i) = val.to_int() {
                    i as f64
                } else if let Some(f) = ctx.get_number(val) {
                    f
                } else {
                    f64::NAN
                };

                // Push original value (this is what the expression returns)
                self.value_stack.push(val)
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;

                // The decremented value should be stored by the calling code
                Ok(None)
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

    fn to_boolean(&self, ctx: &Context, val: JSValue) -> bool {
        use crate::runtime::conversion;
        conversion::to_boolean(ctx, val)
    }

    fn typeof_value(&self, ctx: &Context, val: JSValue) -> &'static str {
        if val.is_undefined() {
            "undefined"
        } else if val.is_null() {
            "object" // typeof null === "object" in JavaScript
        } else if val.is_bool() {
            "boolean"
        } else if val.is_int() {
            "number"
        } else if val.is_ptr() {
            // Check memory tag for function types
            if let Some(index) = val.to_ptr() {
                use crate::memory::MemTag;
                unsafe {
                    let header = ctx.arena().get_header(index);
                    match header.mtag() {
                        MemTag::CFunctionData | MemTag::ClosureData | MemTag::FunctionBytecode => {
                            return "function";
                        }
                        MemTag::String => {
                            return "string";
                        }
                        _ => {}
                    }
                }
            }
            "object"
        } else {
            "undefined"
        }
    }

    // Arithmetic operators (with type coercion)
    fn op_add(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        use crate::runtime::operators;
        operators::add(ctx, a, b).map_err(|_| JSValue::undefined())
    }

    fn op_sub(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        use crate::runtime::operators;
        operators::subtract(ctx, a, b).map_err(|_| JSValue::undefined())
    }

    fn op_mul(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        use crate::runtime::operators;
        operators::multiply(ctx, a, b).map_err(|_| JSValue::undefined())
    }

    fn op_div(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        use crate::runtime::operators;
        operators::divide(ctx, a, b).map_err(|_| JSValue::undefined())
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

    // Comparison operators (with type coercion)
    fn op_lt(&self, ctx: &Context, a: JSValue, b: JSValue) -> Result<bool, JSValue> {
        use crate::runtime::compare;
        Ok(compare::less_than(ctx, a, b))
    }

    fn op_lte(&self, ctx: &Context, a: JSValue, b: JSValue) -> Result<bool, JSValue> {
        use crate::runtime::compare;
        // a <= b is equivalent to !(a > b)
        Ok(!compare::less_than(ctx, b, a))
    }

    fn op_gt(&self, ctx: &Context, a: JSValue, b: JSValue) -> Result<bool, JSValue> {
        use crate::runtime::compare;
        // a > b is equivalent to b < a
        Ok(compare::less_than(ctx, b, a))
    }

    fn op_gte(&self, ctx: &Context, a: JSValue, b: JSValue) -> Result<bool, JSValue> {
        use crate::runtime::compare;
        // a >= b is equivalent to !(a < b)
        Ok(!compare::less_than(ctx, a, b))
    }

    fn op_eq(&self, ctx: &Context, a: JSValue, b: JSValue) -> bool {
        use crate::runtime::compare;
        compare::abstract_equal(ctx, a, b)
    }

    fn op_strict_eq(&self, ctx: &Context, a: JSValue, b: JSValue) -> bool {
        use crate::runtime::compare;
        compare::strict_equal(ctx, a, b)
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

    /// Executes a bytecode function
    ///
    /// This is a simplified implementation that executes the function's bytecode
    /// with local variables stored on the value stack.
    fn execute_bytecode_function(
        &mut self,
        ctx: &mut Context,
        bytecode_index: HeapIndex,
        base_sp: usize,
        local_count: usize,
        closure: Option<HeapIndex>,
    ) -> VMResult {
        // Get the bytecode array
        let bytecode_ptr: *const crate::value::JSByteArray = match ctx.get_byte_array(bytecode_index) {
            Some(b) => b as *const _,
            None => return Err(self.throw_error(ctx, "Invalid function bytecode")),
        };

        // SAFETY: bytecode_ptr is valid as long as we don't modify the arena
        let bytecode_slice = unsafe { (*bytecode_ptr).as_slice() };

        // Parse the function's own constant pool and atom table
        // (Each function has its own embedded tables)
        // For now, we'll create a new bytecode reader from the raw code

        // Skip the constant pool, atom table, and function table headers
        // Since function bytecode is a complete standalone bytecode unit,
        // we need to parse it like a mini-program

        // This is complex - for now, let's use a simplified approach:
        // Execute with a new reader but reuse our stacks
        let mut reader = BytecodeReader::new(bytecode_slice);

        // We need to offset all GetLoc/PutLoc operations by base_sp
        // For now, let's execute the bytecode directly (simplified)
        // The local variable operations will need to be adjusted

        // Actually, we already have locals on the stack at base_sp
        // So we just need to execute the bytecode and intercept GetLoc/SetLoc

        // For simplicity, let's parse the minimal headers and execute
        self.execute_function_bytecode(ctx, &mut reader, base_sp, closure)
    }

    /// Executes function bytecode with proper local variable handling
    fn execute_function_bytecode(
        &mut self,
        ctx: &mut Context,
        reader: &mut BytecodeReader,
        base_sp: usize,
        closure: Option<HeapIndex>,
    ) -> VMResult {
        // Parse headers (constants, atoms, functions)
        // Function bytecode has the same format as main bytecode:
        // [const_count: u16][constants...][atom_count: u16][atoms...][func_count: u16][funcs...][code]

        // Save the current tables so we can restore them after
        let old_constants = core::mem::take(&mut self.constants);
        let old_const_is_f64 = core::mem::take(&mut self.const_is_f64);
        let old_atom_table = core::mem::take(&mut self.atom_table);
        let old_function_table = core::mem::take(&mut self.function_table);

        // Parse constant pool (same format as main bytecode: type byte + raw JSValue)
        // Type: 0 = f64 bits, 1 = JSValue
        let const_count = {
            let byte0 = reader.read_u8().unwrap_or(0);
            let byte1 = reader.read_u8().unwrap_or(0);
            u16::from_le_bytes([byte0, byte1]) as usize
        };

        self.constants = alloc::vec::Vec::with_capacity(const_count);
        self.const_is_f64 = alloc::vec::Vec::with_capacity(const_count);
        for _ in 0..const_count {
            let const_type = reader.read_u8().unwrap_or(0);
            let mut value_bytes = [0u8; core::mem::size_of::<usize>()];
            for i in 0..core::mem::size_of::<usize>() {
                value_bytes[i] = reader.read_u8().unwrap_or(0);
            }
            let raw = usize::from_le_bytes(value_bytes);
            let value = unsafe { core::mem::transmute::<usize, JSValue>(raw) };
            self.constants.push(value);
            self.const_is_f64.push(const_type == 0);
        }

        // Parse atom table
        let atom_count = {
            let byte0 = reader.read_u8().unwrap_or(0);
            let byte1 = reader.read_u8().unwrap_or(0);
            u16::from_le_bytes([byte0, byte1]) as usize
        };

        self.atom_table = alloc::vec::Vec::with_capacity(atom_count);
        for _ in 0..atom_count {
            let len = {
                let byte0 = reader.read_u8().unwrap_or(0);
                let byte1 = reader.read_u8().unwrap_or(0);
                u16::from_le_bytes([byte0, byte1]) as usize
            };
            let mut name_bytes = alloc::vec::Vec::with_capacity(len);
            for _ in 0..len {
                name_bytes.push(reader.read_u8().unwrap_or(0));
            }
            let name = alloc::string::String::from_utf8(name_bytes)
                .unwrap_or_else(|_| alloc::string::String::new());
            self.atom_table.push(name);
        }

        // Parse function table
        let func_count = {
            let byte0 = reader.read_u8().unwrap_or(0);
            let byte1 = reader.read_u8().unwrap_or(0);
            u16::from_le_bytes([byte0, byte1]) as usize
        };

        self.function_table = alloc::vec::Vec::with_capacity(func_count);
        for _ in 0..func_count {
            let param_count = reader.read_u8().unwrap_or(0);
            let local_count = reader.read_u8().unwrap_or(0);
            let bytecode_len = {
                let mut bytes = [0u8; 4];
                for i in 0..4 {
                    bytes[i] = reader.read_u8().unwrap_or(0);
                }
                u32::from_le_bytes(bytes) as usize
            };

            // Allocate the bytecode on the heap
            let bytecode_index = match ctx.alloc_byte_array(bytecode_len) {
                Ok(idx) => idx,
                Err(_) => {
                    // Restore tables and return error
                    self.constants = old_constants;
                    self.const_is_f64 = old_const_is_f64;
                    self.atom_table = old_atom_table;
                    self.function_table = old_function_table;
                    return Err(self.throw_error(ctx, "Out of memory loading function bytecode"));
                }
            };

            // Read the bytecode directly into the allocated array
            if let Some(array) = ctx.get_byte_array_mut(bytecode_index) {
                // SAFETY: We just allocated the array with bytecode_len capacity
                let slice = unsafe { array.as_full_mut_slice() };
                for i in 0..bytecode_len {
                    if i < slice.len() {
                        slice[i] = reader.read_u8().unwrap_or(0);
                    } else {
                        reader.read_u8(); // Skip if array is too small (shouldn't happen)
                    }
                }
                // Set the count so as_slice() works correctly
                array.header_mut().set_count(bytecode_len);
            } else {
                // Skip the bytecode bytes
                for _ in 0..bytecode_len {
                    reader.read_u8();
                }
            }

            self.function_table.push(FunctionEntry {
                bytecode_index,
                param_count,
                local_count,
            });
        }

        // Execute the actual code
        let result = self.execute_function_code(ctx, reader, base_sp, closure);

        // Restore the old tables
        self.constants = old_constants;
        self.const_is_f64 = old_const_is_f64;
        self.atom_table = old_atom_table;
        self.function_table = old_function_table;

        result
    }

    /// Inner execution loop for function bytecode
    fn execute_function_code(
        &mut self,
        ctx: &mut Context,
        reader: &mut BytecodeReader,
        base_sp: usize,
        closure: Option<HeapIndex>,
    ) -> VMResult {
        loop {
            let instruction = match reader.decode() {
                Some(inst) => inst,
                None => return Ok(JSValue::undefined()),
            };

            // Handle local variable access and closure variable access specially
            match instruction.opcode {
                Opcode::GetLoc => {
                    if let Operand::U8(idx) = instruction.operand {
                        let local_val = self.value_stack.get(base_sp + idx as usize)
                            .map_err(|_| self.throw_error(ctx, "Invalid local variable index"))?;
                        self.value_stack.push(local_val)
                            .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    }
                }
                Opcode::PutLoc => {
                    if let Operand::U8(idx) = instruction.operand {
                        let val = self.value_stack.pop()
                            .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                        self.value_stack.set(base_sp + idx as usize, val)
                            .map_err(|_| self.throw_error(ctx, "Invalid local variable index"))?;
                    }
                }
                Opcode::SetLoc => {
                    if let Operand::U8(idx) = instruction.operand {
                        let val = self.value_stack.peek()
                            .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                        self.value_stack.set(base_sp + idx as usize, val)
                            .map_err(|_| self.throw_error(ctx, "Invalid local variable index"))?;
                    }
                }
                Opcode::GetVarRef => {
                    // Get a captured variable from the closure's environment
                    if let Operand::U8(var_idx) = instruction.operand {
                        let closure_idx = match closure {
                            Some(idx) => idx,
                            None => return Err(self.throw_error(ctx, "GetVarRef outside closure")),
                        };

                        let var_ref_idx = match ctx.get_closure(closure_idx) {
                            Some(c) => {
                                if var_idx >= c.var_ref_count {
                                    return Err(self.throw_error(ctx, "Var ref index out of bounds"));
                                }
                                c.get_var_ref(var_idx as usize)
                            }
                            None => return Err(self.throw_error(ctx, "Invalid closure")),
                        };

                        let value = match ctx.get_var_ref(var_ref_idx) {
                            Some(var_ref) => var_ref.value(),
                            None => return Err(self.throw_error(ctx, "Invalid var ref")),
                        };

                        self.value_stack.push(value)
                            .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                    }
                }
                Opcode::PutVarRef => {
                    // Set a captured variable (pops value from stack)
                    if let Operand::U8(var_idx) = instruction.operand {
                        let value = self.value_stack.pop()
                            .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                        let closure_idx = match closure {
                            Some(idx) => idx,
                            None => return Err(self.throw_error(ctx, "PutVarRef outside closure")),
                        };

                        let var_ref_idx = match ctx.get_closure(closure_idx) {
                            Some(c) => {
                                if var_idx >= c.var_ref_count {
                                    return Err(self.throw_error(ctx, "Var ref index out of bounds"));
                                }
                                c.get_var_ref(var_idx as usize)
                            }
                            None => return Err(self.throw_error(ctx, "Invalid closure")),
                        };

                        match ctx.get_var_ref_mut(var_ref_idx) {
                            Some(var_ref) => var_ref.set_value(value),
                            None => return Err(self.throw_error(ctx, "Invalid var ref")),
                        }
                    }
                }
                Opcode::SetVarRef => {
                    // Set a captured variable (leaves value on stack)
                    if let Operand::U8(var_idx) = instruction.operand {
                        let value = self.value_stack.peek()
                            .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;

                        let closure_idx = match closure {
                            Some(idx) => idx,
                            None => return Err(self.throw_error(ctx, "SetVarRef outside closure")),
                        };

                        let var_ref_idx = match ctx.get_closure(closure_idx) {
                            Some(c) => {
                                if var_idx >= c.var_ref_count {
                                    return Err(self.throw_error(ctx, "Var ref index out of bounds"));
                                }
                                c.get_var_ref(var_idx as usize)
                            }
                            None => return Err(self.throw_error(ctx, "Invalid closure")),
                        };

                        match ctx.get_var_ref_mut(var_ref_idx) {
                            Some(var_ref) => var_ref.set_value(value),
                            None => return Err(self.throw_error(ctx, "Invalid var ref")),
                        }
                    }
                }
                Opcode::Return => {
                    let ret_val = self.value_stack.pop()
                        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
                    return Ok(ret_val);
                }
                Opcode::ReturnUndef => {
                    return Ok(JSValue::undefined());
                }
                _ => {
                    // For other opcodes, execute normally
                    match self.execute_instruction(ctx, reader, &instruction)? {
                        Some(ret) => return Ok(ret),
                        None => continue,
                    }
                }
            }
        }
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

        // Add headers: [const_count: u16][constants...][atom_count: u16][atoms...][func_count: u16][funcs...][bytecode...]
        let mut bytecode = Vec::new();
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 constants
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 atoms
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 functions
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

        // Add headers
        let mut bytecode = Vec::new();
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 constants
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 atoms
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 functions
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

        // Add headers
        let mut bytecode = Vec::new();
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 constants
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 atoms
        bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 functions
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
