//! Bytecode interpreter

use crate::value::JSValue;

/// Virtual machine state
pub struct VM {
    // TODO: Implement fields:
    // - stack: Stack
    // - pc: usize (program counter)
    // - fp: usize (frame pointer)
    _placeholder: u8,
}

impl VM {
    /// Creates a new VM
    pub fn new() -> Self {
        VM {
            _placeholder: 0,
        }
    }

    /// Executes bytecode
    pub fn execute(&mut self, _bytecode: &[u8]) -> JSValue {
        // TODO: Dispatch loop
        // TODO: Decode and execute opcodes
        JSValue::undefined()
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}
