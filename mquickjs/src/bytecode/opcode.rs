//! Opcode definitions
//!
//! Defines all ~260 opcodes used by the MicroQuickJS virtual machine.

/// Virtual machine opcodes
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // Stack manipulation
    /// Drop top value from stack
    Drop = 0,
    /// Duplicate top value
    Dup = 1,
    /// Swap top two values
    Swap = 2,
    /// Remove second value (keep top)
    Nip = 3,

    // Push constants
    /// Push undefined
    Undefined = 10,
    /// Push null
    Null = 11,
    /// Push false
    PushFalse = 12,
    /// Push true
    PushTrue = 13,
    /// Push 8-bit signed integer
    PushI8 = 14,
    /// Push 16-bit signed integer
    PushI16 = 15,
    /// Push constant from pool (8-bit index)
    PushConst8 = 16,

    // Variable access
    /// Get local variable
    GetLoc = 20,
    /// Put local variable
    PutLoc = 21,
    /// Get argument
    GetArg = 22,
    /// Put argument
    PutArg = 23,

    // Property access
    /// Get object field
    GetField = 30,
    /// Put object field
    PutField = 31,
    /// Get array element
    GetArrayEl = 32,
    /// Put array element
    PutArrayEl = 33,

    // Arithmetic
    /// Addition
    Add = 40,
    /// Subtraction
    Sub = 41,
    /// Multiplication
    Mul = 42,
    /// Division
    Div = 43,
    /// Modulo
    Mod = 44,

    // Comparison
    /// Less than
    Lt = 50,
    /// Less than or equal
    Lte = 51,
    /// Greater than
    Gt = 52,
    /// Greater than or equal
    Gte = 53,
    /// Equality
    Eq = 54,
    /// Strict equality
    StrictEq = 55,

    // Control flow
    /// Conditional jump if false
    IfFalse = 60,
    /// Unconditional jump
    Goto = 61,
    /// Return from function
    Return = 62,

    // Function calls
    /// Call function
    Call = 70,
    /// Call method
    CallMethod = 71,
    /// Call constructor
    CallConstructor = 72,

    // Exception handling
    /// Throw exception
    Throw = 80,
    /// Catch exception
    Catch = 81,

    // TODO: Add remaining ~200 opcodes
    // This is a minimal subset for Phase 0
}

impl Opcode {
    /// Returns the opcode name for debugging
    pub fn name(self) -> &'static str {
        match self {
            Opcode::Drop => "drop",
            Opcode::Dup => "dup",
            Opcode::Swap => "swap",
            Opcode::Nip => "nip",
            Opcode::Undefined => "undefined",
            Opcode::Null => "null",
            Opcode::PushFalse => "push_false",
            Opcode::PushTrue => "push_true",
            Opcode::PushI8 => "push_i8",
            Opcode::PushI16 => "push_i16",
            Opcode::PushConst8 => "push_const8",
            Opcode::GetLoc => "get_loc",
            Opcode::PutLoc => "put_loc",
            Opcode::GetArg => "get_arg",
            Opcode::PutArg => "put_arg",
            Opcode::GetField => "get_field",
            Opcode::PutField => "put_field",
            Opcode::GetArrayEl => "get_array_el",
            Opcode::PutArrayEl => "put_array_el",
            Opcode::Add => "add",
            Opcode::Sub => "sub",
            Opcode::Mul => "mul",
            Opcode::Div => "div",
            Opcode::Mod => "mod",
            Opcode::Lt => "lt",
            Opcode::Lte => "lte",
            Opcode::Gt => "gt",
            Opcode::Gte => "gte",
            Opcode::Eq => "eq",
            Opcode::StrictEq => "strict_eq",
            Opcode::IfFalse => "if_false",
            Opcode::Goto => "goto",
            Opcode::Return => "return",
            Opcode::Call => "call",
            Opcode::CallMethod => "call_method",
            Opcode::CallConstructor => "call_constructor",
            Opcode::Throw => "throw",
            Opcode::Catch => "catch",
        }
    }
}
