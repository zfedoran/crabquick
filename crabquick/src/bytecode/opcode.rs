//! Opcode definitions
//!
//! Defines all ~260 opcodes used by the MicroQuickJS virtual machine.
//! Based on the MicroQuickJS C implementation.

#![allow(dead_code)]

/// Virtual machine opcodes
///
/// These opcodes form the instruction set for the MicroQuickJS bytecode interpreter.
/// The opcodes are organized into categories:
/// - Stack manipulation (drop, dup, swap, etc.)
/// - Push operations (constants, literals)
/// - Variable access (local, argument, closure vars)
/// - Property access (object fields, array elements)
/// - Arithmetic operations
/// - Comparison operations
/// - Logical operations
/// - Bitwise operations
/// - Control flow (jumps, calls, returns)
/// - Object operations
/// - Exception handling
/// - Special operations
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // ===== Stack Manipulation =====
    /// Drop top value from stack
    Drop = 0,
    /// Duplicate top value
    Dup = 1,
    /// Swap top two values
    Swap = 2,
    /// Remove second value (keep top)
    Nip = 3,
    /// Insert third value at top (a b c -> c a b)
    Insert2 = 4,
    /// Insert fourth value at top (a b c d -> d a b c)
    Insert3 = 5,
    /// Permute top 3 values (a b c -> c b a)
    Perm3 = 6,
    /// Rotate top 3 values left (a b c -> b c a)
    Rot3l = 7,
    /// Rotate top 3 values right (a b c -> c a b)
    Rot3r = 8,
    /// Rotate top 4 values left (a b c d -> b c d a)
    Rot4l = 9,

    // ===== Push Operations =====
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
    /// Push 32-bit signed integer
    PushI32 = 16,
    /// Push constant from pool (8-bit index)
    PushConst8 = 17,
    /// Push constant from pool (16-bit index)
    PushConst16 = 18,
    /// Push -1
    PushMinus1 = 19,
    /// Push 0
    Push0 = 20,
    /// Push 1
    Push1 = 21,
    /// Push 2
    Push2 = 22,
    /// Push 3
    Push3 = 23,
    /// Push 4
    Push4 = 24,
    /// Push 5
    Push5 = 25,
    /// Push 6
    Push6 = 26,
    /// Push 7
    Push7 = 27,
    /// Push empty string
    PushEmptyString = 28,
    /// Push this
    PushThis = 29,
    /// Push NaN
    PushNaN = 30,
    /// Push +Infinity
    PushInfinity = 31,
    /// Push -Infinity
    PushNegInfinity = 32,
    /// Push function from function table (8-bit index)
    PushFunc8 = 33,
    /// Push function from function table (16-bit index)
    PushFunc = 34,
    /// Push string from atom table (8-bit index)
    PushAtomString8 = 35,
    /// Push string from atom table (16-bit index)
    PushAtomString16 = 36,

    // ===== Variable Access =====
    /// Get local variable
    GetLoc = 40,
    /// Put local variable
    PutLoc = 41,
    /// Set local variable (returns value)
    SetLoc = 42,
    /// Get argument
    GetArg = 43,
    /// Put argument
    PutArg = 44,
    /// Set argument (returns value)
    SetArg = 45,
    /// Get var ref (closure variable)
    GetVarRef = 46,
    /// Put var ref (closure variable)
    PutVarRef = 47,
    /// Set var ref (returns value)
    SetVarRef = 48,
    /// Get local 0 (fast path)
    GetLoc0 = 49,
    /// Get local 1 (fast path)
    GetLoc1 = 50,
    /// Get local 2 (fast path)
    GetLoc2 = 51,
    /// Get local 3 (fast path)
    GetLoc3 = 52,
    /// Put local 0 (fast path)
    PutLoc0 = 53,
    /// Put local 1 (fast path)
    PutLoc1 = 54,
    /// Put local 2 (fast path)
    PutLoc2 = 55,
    /// Put local 3 (fast path)
    PutLoc3 = 56,
    /// Set local 0 (fast path)
    SetLoc0 = 57,
    /// Set local 1 (fast path)
    SetLoc1 = 58,
    /// Set local 2 (fast path)
    SetLoc2 = 59,
    /// Set local 3 (fast path)
    SetLoc3 = 60,
    /// Get global variable (8-bit atom index)
    GetGlobal8 = 61,
    /// Get global variable (16-bit atom index)
    GetGlobal16 = 62,
    /// Put global variable (8-bit atom index)
    PutGlobal8 = 63,
    /// Put global variable (16-bit atom index)
    PutGlobal16 = 64,
    /// Set global variable (returns value, 8-bit atom index)
    SetGlobal8 = 65,
    /// Set global variable (returns value, 16-bit atom index)
    SetGlobal16 = 66,

    // ===== Property Access =====
    /// Get object field
    GetField = 70,
    /// Get object field (8-bit atom index)
    GetField8 = 71,
    /// Put object field
    PutField = 72,
    /// Put object field (8-bit atom index)
    PutField8 = 73,
    /// Get private field
    GetPrivateField = 74,
    /// Put private field
    PutPrivateField = 75,
    /// Define object field
    DefineField = 76,
    /// Set object field (returns value)
    SetField = 77,
    /// Get array element
    GetArrayEl = 78,
    /// Put array element
    PutArrayEl = 79,
    /// Get super value
    GetSuper = 80,
    /// Put super value
    PutSuper = 81,
    /// Define array element
    DefineArrayEl = 82,
    /// Set super value
    SetSuper = 83,
    /// Set array element (returns value)
    SetArrayEl = 84,
    /// Get array length
    GetLength = 85,

    // ===== Arithmetic Operations =====
    /// Addition
    Add = 90,
    /// Subtraction
    Sub = 91,
    /// Multiplication
    Mul = 92,
    /// Division
    Div = 93,
    /// Modulo
    Mod = 94,
    /// Exponentiation
    Pow = 95,
    /// Unary plus
    Plus = 96,
    /// Unary minus
    Neg = 97,
    /// Increment
    Inc = 98,
    /// Decrement
    Dec = 99,
    /// Post increment
    PostInc = 100,
    /// Post decrement
    PostDec = 101,

    // ===== Comparison Operations =====
    /// Less than
    Lt = 110,
    /// Less than or equal
    Lte = 111,
    /// Greater than
    Gt = 112,
    /// Greater than or equal
    Gte = 113,
    /// Equality (==)
    Eq = 114,
    /// Inequality (!=)
    Neq = 115,
    /// Strict equality (===)
    StrictEq = 116,
    /// Strict inequality (!==)
    StrictNeq = 117,
    /// instanceof
    Instanceof = 118,
    /// in
    In = 119,

    // ===== Logical Operations =====
    /// Logical NOT
    LNot = 130,
    /// Logical AND (short-circuit)
    LAnd = 131,
    /// Logical OR (short-circuit)
    LOr = 132,
    /// Nullish coalescing (??)
    Nullish = 133,

    // ===== Bitwise Operations =====
    /// Bitwise NOT
    Not = 140,
    /// Bitwise AND
    And = 141,
    /// Bitwise OR
    Or = 142,
    /// Bitwise XOR
    Xor = 143,
    /// Left shift (<<)
    Shl = 144,
    /// Signed right shift (>>)
    Sar = 145,
    /// Unsigned right shift (>>>)
    Shr = 146,

    // ===== Control Flow =====
    /// Conditional jump if false
    IfFalse = 160,
    /// Conditional jump if true
    IfTrue = 161,
    /// Unconditional jump
    Goto = 162,
    /// Return from function
    Return = 163,
    /// Return undefined
    ReturnUndef = 164,
    /// Gosub (for finally blocks)
    Gosub = 165,
    /// Return from gosub
    Ret = 166,
    /// Check variable initialized
    CheckVar = 167,
    /// Check `this` not undefined
    CheckThis = 168,
    /// Break with label
    Break = 169,
    /// Continue with label
    Continue = 170,

    // ===== Function Calls =====
    /// Call function (argc on stack)
    Call = 180,
    /// Tail call function
    TailCall = 181,
    /// Call method
    CallMethod = 182,
    /// Tail call method
    TailCallMethod = 183,
    /// Call constructor (new)
    CallConstructor = 184,
    /// eval() call
    Eval = 185,
    /// Apply (with arguments array)
    Apply = 186,
    /// Apply with eval
    ApplyEval = 187,
    /// Spread call arguments
    CallSpread = 188,

    // ===== Object Operations =====
    /// Create new object
    Object = 200,
    /// Create new array
    Array = 201,
    /// Create regexp
    Regexp = 202,
    /// Get iterator
    GetIterator = 203,
    /// Get async iterator
    GetAsyncIterator = 204,
    /// Iterator next
    IteratorNext = 205,
    /// Iterator close
    IteratorClose = 206,
    /// Iterator check object
    IteratorCheckObject = 207,
    /// for-in start
    ForInStart = 208,
    /// for-in next
    ForInNext = 209,
    /// for-of start
    ForOfStart = 210,
    /// for-of next
    ForOfNext = 211,
    /// typeof operator
    TypeOf = 212,
    /// delete operator
    Delete = 213,
    /// delete var
    DeleteVar = 214,
    /// void operator
    Void = 215,
    /// Spread array element
    SpreadArray = 216,
    /// Spread object properties
    SpreadObject = 217,
    /// Copy data properties
    CopyDataProperties = 218,
    /// Define private field
    DefinePrivateField = 219,
    /// Define method
    DefineMethod = 220,
    /// Define getter
    DefineGetter = 221,
    /// Define setter
    DefineSetter = 222,
    /// Define class name
    DefineClassName = 223,
    /// Create arguments object
    Arguments = 224,
    /// Get rest arguments
    RestArgs = 225,
    /// Define class
    DefineClass = 226,
    /// Set home object
    SetHomeObject = 227,
    /// Set name
    SetName = 228,
    /// Set prototype
    SetProto = 229,

    // ===== Closure Operations =====
    /// Create closure
    FClosure = 240,
    /// Create closure with varargs
    FClosureVarArgs = 241,
    /// Set closure var ref
    SetVarRefThis = 242,
    /// Get closure var ref (checked)
    GetVarRefCheck = 243,
    /// Put closure var ref (checked)
    PutVarRefCheck = 244,
    /// Set closure var ref (checked)
    SetVarRefCheck = 245,

    // ===== Exception Handling =====
    /// Clear catch offset (after try block completes normally)
    ClearCatchOffset = 248,
    /// Throw exception
    Throw = 250,
    /// Throw error (from type)
    ThrowError = 251,
    /// Catch exception
    Catch = 252,
    /// Push catch offset
    PushCatchOffset = 253,
    /// Rethrow exception
    Rethrow = 254,

    // ===== Special Operations =====
    /// No-op
    Nop = 255,
}

/// Instruction format type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InstructionFormat {
    /// No operands
    None,
    /// 1-byte unsigned integer
    U8,
    /// 1-byte signed integer
    I8,
    /// 2-byte unsigned integer
    U16,
    /// 2-byte signed integer
    I16,
    /// 4-byte unsigned integer
    U32,
    /// 4-byte signed integer
    I32,
    /// 4-byte jump label (PC offset)
    Label,
    /// 1-byte constant pool index
    Const8,
    /// 2-byte constant pool index
    Const16,
    /// 1-byte atom index
    Atom8,
    /// 2-byte atom index
    Atom16,
}

impl Opcode {
    /// Returns the opcode name for debugging
    pub fn name(self) -> &'static str {
        match self {
            // Stack manipulation
            Opcode::Drop => "drop",
            Opcode::Dup => "dup",
            Opcode::Swap => "swap",
            Opcode::Nip => "nip",
            Opcode::Insert2 => "insert2",
            Opcode::Insert3 => "insert3",
            Opcode::Perm3 => "perm3",
            Opcode::Rot3l => "rot3l",
            Opcode::Rot3r => "rot3r",
            Opcode::Rot4l => "rot4l",

            // Push operations
            Opcode::Undefined => "undefined",
            Opcode::Null => "null",
            Opcode::PushFalse => "push_false",
            Opcode::PushTrue => "push_true",
            Opcode::PushI8 => "push_i8",
            Opcode::PushI16 => "push_i16",
            Opcode::PushI32 => "push_i32",
            Opcode::PushConst8 => "push_const8",
            Opcode::PushConst16 => "push_const16",
            Opcode::PushMinus1 => "push_minus1",
            Opcode::Push0 => "push_0",
            Opcode::Push1 => "push_1",
            Opcode::Push2 => "push_2",
            Opcode::Push3 => "push_3",
            Opcode::Push4 => "push_4",
            Opcode::Push5 => "push_5",
            Opcode::Push6 => "push_6",
            Opcode::Push7 => "push_7",
            Opcode::PushEmptyString => "push_empty_string",
            Opcode::PushThis => "push_this",
            Opcode::PushNaN => "push_nan",
            Opcode::PushInfinity => "push_infinity",
            Opcode::PushNegInfinity => "push_neg_infinity",
            Opcode::PushFunc8 => "push_func8",
            Opcode::PushFunc => "push_func",
            Opcode::PushAtomString8 => "push_atom_string8",
            Opcode::PushAtomString16 => "push_atom_string16",

            // Variable access
            Opcode::GetLoc => "get_loc",
            Opcode::PutLoc => "put_loc",
            Opcode::SetLoc => "set_loc",
            Opcode::GetArg => "get_arg",
            Opcode::PutArg => "put_arg",
            Opcode::SetArg => "set_arg",
            Opcode::GetVarRef => "get_var_ref",
            Opcode::PutVarRef => "put_var_ref",
            Opcode::SetVarRef => "set_var_ref",
            Opcode::GetLoc0 => "get_loc0",
            Opcode::GetLoc1 => "get_loc1",
            Opcode::GetLoc2 => "get_loc2",
            Opcode::GetLoc3 => "get_loc3",
            Opcode::PutLoc0 => "put_loc0",
            Opcode::PutLoc1 => "put_loc1",
            Opcode::PutLoc2 => "put_loc2",
            Opcode::PutLoc3 => "put_loc3",
            Opcode::SetLoc0 => "set_loc0",
            Opcode::SetLoc1 => "set_loc1",
            Opcode::SetLoc2 => "set_loc2",
            Opcode::SetLoc3 => "set_loc3",
            Opcode::GetGlobal8 => "get_global8",
            Opcode::GetGlobal16 => "get_global16",
            Opcode::PutGlobal8 => "put_global8",
            Opcode::PutGlobal16 => "put_global16",
            Opcode::SetGlobal8 => "set_global8",
            Opcode::SetGlobal16 => "set_global16",

            // Property access
            Opcode::GetField => "get_field",
            Opcode::GetField8 => "get_field8",
            Opcode::PutField => "put_field",
            Opcode::PutField8 => "put_field8",
            Opcode::GetPrivateField => "get_private_field",
            Opcode::PutPrivateField => "put_private_field",
            Opcode::DefineField => "define_field",
            Opcode::SetField => "set_field",
            Opcode::GetArrayEl => "get_array_el",
            Opcode::PutArrayEl => "put_array_el",
            Opcode::GetSuper => "get_super",
            Opcode::PutSuper => "put_super",
            Opcode::DefineArrayEl => "define_array_el",
            Opcode::SetSuper => "set_super",
            Opcode::SetArrayEl => "set_array_el",
            Opcode::GetLength => "get_length",

            // Arithmetic
            Opcode::Add => "add",
            Opcode::Sub => "sub",
            Opcode::Mul => "mul",
            Opcode::Div => "div",
            Opcode::Mod => "mod",
            Opcode::Pow => "pow",
            Opcode::Plus => "plus",
            Opcode::Neg => "neg",
            Opcode::Inc => "inc",
            Opcode::Dec => "dec",
            Opcode::PostInc => "post_inc",
            Opcode::PostDec => "post_dec",

            // Comparison
            Opcode::Lt => "lt",
            Opcode::Lte => "lte",
            Opcode::Gt => "gt",
            Opcode::Gte => "gte",
            Opcode::Eq => "eq",
            Opcode::Neq => "neq",
            Opcode::StrictEq => "strict_eq",
            Opcode::StrictNeq => "strict_neq",
            Opcode::Instanceof => "instanceof",
            Opcode::In => "in",

            // Logical
            Opcode::LNot => "lnot",
            Opcode::LAnd => "land",
            Opcode::LOr => "lor",
            Opcode::Nullish => "nullish",

            // Bitwise
            Opcode::Not => "not",
            Opcode::And => "and",
            Opcode::Or => "or",
            Opcode::Xor => "xor",
            Opcode::Shl => "shl",
            Opcode::Sar => "sar",
            Opcode::Shr => "shr",

            // Control flow
            Opcode::IfFalse => "if_false",
            Opcode::IfTrue => "if_true",
            Opcode::Goto => "goto",
            Opcode::Return => "return",
            Opcode::ReturnUndef => "return_undef",
            Opcode::Gosub => "gosub",
            Opcode::Ret => "ret",
            Opcode::CheckVar => "check_var",
            Opcode::CheckThis => "check_this",
            Opcode::Break => "break",
            Opcode::Continue => "continue",

            // Function calls
            Opcode::Call => "call",
            Opcode::TailCall => "tail_call",
            Opcode::CallMethod => "call_method",
            Opcode::TailCallMethod => "tail_call_method",
            Opcode::CallConstructor => "call_constructor",
            Opcode::Eval => "eval",
            Opcode::Apply => "apply",
            Opcode::ApplyEval => "apply_eval",
            Opcode::CallSpread => "call_spread",

            // Object operations
            Opcode::Object => "object",
            Opcode::Array => "array",
            Opcode::Regexp => "regexp",
            Opcode::GetIterator => "get_iterator",
            Opcode::GetAsyncIterator => "get_async_iterator",
            Opcode::IteratorNext => "iterator_next",
            Opcode::IteratorClose => "iterator_close",
            Opcode::IteratorCheckObject => "iterator_check_object",
            Opcode::ForInStart => "for_in_start",
            Opcode::ForInNext => "for_in_next",
            Opcode::ForOfStart => "for_of_start",
            Opcode::ForOfNext => "for_of_next",
            Opcode::TypeOf => "typeof",
            Opcode::Delete => "delete",
            Opcode::DeleteVar => "delete_var",
            Opcode::Void => "void",
            Opcode::SpreadArray => "spread_array",
            Opcode::SpreadObject => "spread_object",
            Opcode::CopyDataProperties => "copy_data_properties",
            Opcode::DefinePrivateField => "define_private_field",
            Opcode::DefineMethod => "define_method",
            Opcode::DefineGetter => "define_getter",
            Opcode::DefineSetter => "define_setter",
            Opcode::DefineClassName => "define_class_name",
            Opcode::Arguments => "arguments",
            Opcode::RestArgs => "rest_args",
            Opcode::DefineClass => "define_class",
            Opcode::SetHomeObject => "set_home_object",
            Opcode::SetName => "set_name",
            Opcode::SetProto => "set_proto",

            // Closure operations
            Opcode::FClosure => "fclosure",
            Opcode::FClosureVarArgs => "fclosure_varargs",
            Opcode::SetVarRefThis => "set_var_ref_this",
            Opcode::GetVarRefCheck => "get_var_ref_check",
            Opcode::PutVarRefCheck => "put_var_ref_check",
            Opcode::SetVarRefCheck => "set_var_ref_check",

            // Exception handling
            Opcode::ClearCatchOffset => "clear_catch_offset",
            Opcode::Throw => "throw",
            Opcode::ThrowError => "throw_error",
            Opcode::Catch => "catch",
            Opcode::PushCatchOffset => "push_catch_offset",
            Opcode::Rethrow => "rethrow",

            // Special
            Opcode::Nop => "nop",
        }
    }

    /// Returns the instruction format for this opcode
    pub fn format(self) -> InstructionFormat {
        use InstructionFormat::*;

        match self {
            // No operands
            Opcode::Drop | Opcode::Dup | Opcode::Swap | Opcode::Nip |
            Opcode::Insert2 | Opcode::Insert3 | Opcode::Perm3 | Opcode::Rot3l |
            Opcode::Rot3r | Opcode::Rot4l |
            Opcode::Undefined | Opcode::Null | Opcode::PushFalse | Opcode::PushTrue |
            Opcode::PushMinus1 | Opcode::Push0 | Opcode::Push1 | Opcode::Push2 |
            Opcode::Push3 | Opcode::Push4 | Opcode::Push5 | Opcode::Push6 |
            Opcode::Push7 | Opcode::PushEmptyString | Opcode::PushThis |
            Opcode::PushNaN | Opcode::PushInfinity | Opcode::PushNegInfinity |
            Opcode::GetLoc0 | Opcode::GetLoc1 | Opcode::GetLoc2 | Opcode::GetLoc3 |
            Opcode::PutLoc0 | Opcode::PutLoc1 | Opcode::PutLoc2 | Opcode::PutLoc3 |
            Opcode::SetLoc0 | Opcode::SetLoc1 | Opcode::SetLoc2 | Opcode::SetLoc3 |
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod |
            Opcode::Pow | Opcode::Plus | Opcode::Neg | Opcode::Inc | Opcode::Dec |
            Opcode::PostInc | Opcode::PostDec |
            Opcode::Lt | Opcode::Lte | Opcode::Gt | Opcode::Gte |
            Opcode::Eq | Opcode::Neq | Opcode::StrictEq | Opcode::StrictNeq |
            Opcode::Instanceof | Opcode::In |
            Opcode::LNot | Opcode::Not |
            Opcode::And | Opcode::Or | Opcode::Xor |
            Opcode::Shl | Opcode::Sar | Opcode::Shr |
            Opcode::Return | Opcode::ReturnUndef |
            Opcode::GetArrayEl | Opcode::PutArrayEl | Opcode::SetArrayEl |
            Opcode::GetLength | Opcode::ClearCatchOffset | Opcode::Throw | Opcode::Catch | Opcode::Rethrow |
            Opcode::GetIterator | Opcode::GetAsyncIterator | Opcode::IteratorNext |
            Opcode::IteratorClose | Opcode::IteratorCheckObject |
            Opcode::TypeOf | Opcode::Delete | Opcode::DeleteVar | Opcode::Void |
            Opcode::ForInStart | Opcode::ForInNext |
            Opcode::ForOfStart | Opcode::ForOfNext |
            Opcode::Nop => None,

            // U8 operands
            Opcode::GetLoc | Opcode::PutLoc | Opcode::SetLoc |
            Opcode::GetArg | Opcode::PutArg | Opcode::SetArg |
            Opcode::GetVarRef | Opcode::PutVarRef | Opcode::SetVarRef |
            Opcode::Call | Opcode::TailCall |
            Opcode::CallMethod | Opcode::TailCallMethod |
            Opcode::CallConstructor | Opcode::Apply | Opcode::ApplyEval |
            Opcode::Array | Opcode::Object | Opcode::PushFunc8 => U8,

            // Atom8 operands (for global variable names and string literals)
            Opcode::GetGlobal8 | Opcode::PutGlobal8 | Opcode::SetGlobal8 |
            Opcode::PushAtomString8 => Atom8,

            // Atom16 operands (for global variable names and string literals)
            Opcode::GetGlobal16 | Opcode::PutGlobal16 | Opcode::SetGlobal16 |
            Opcode::PushAtomString16 => Atom16,

            // I8 operands
            Opcode::PushI8 => I8,

            // U16 operands
            Opcode::GetField | Opcode::PutField | Opcode::DefineField | Opcode::SetField |
            Opcode::GetPrivateField | Opcode::PutPrivateField |
            Opcode::GetSuper | Opcode::PutSuper | Opcode::DefineArrayEl | Opcode::SetSuper |
            Opcode::PushFunc | Opcode::DefineGetter | Opcode::DefineSetter => U16,

            // I16 operands
            Opcode::PushI16 => I16,

            // U32 operands
            Opcode::CheckVar | Opcode::CheckThis => U32,

            // I32 operands
            Opcode::PushI32 => I32,

            // Label operands (4-byte PC offset)
            Opcode::IfFalse | Opcode::IfTrue | Opcode::Goto |
            Opcode::Gosub | Opcode::Ret | Opcode::Break | Opcode::Continue |
            Opcode::LAnd | Opcode::LOr | Opcode::Nullish |
            Opcode::PushCatchOffset => Label,

            // Const8 operands
            Opcode::PushConst8 | Opcode::FClosure => Const8,

            // Const16 operands
            Opcode::PushConst16 | Opcode::FClosureVarArgs |
            Opcode::Regexp | Opcode::Eval => Const16,

            // Atom8 operands
            Opcode::GetField8 | Opcode::PutField8 => Atom8,

            // Other special cases
            Opcode::SpreadArray | Opcode::SpreadObject |
            Opcode::CopyDataProperties | Opcode::DefinePrivateField |
            Opcode::DefineMethod |
            Opcode::DefineClassName | Opcode::Arguments | Opcode::RestArgs |
            Opcode::DefineClass | Opcode::SetHomeObject | Opcode::SetName |
            Opcode::SetProto | Opcode::SetVarRefThis |
            Opcode::GetVarRefCheck | Opcode::PutVarRefCheck | Opcode::SetVarRefCheck |
            Opcode::ThrowError | Opcode::CallSpread => U8,
        }
    }

    /// Returns the size of the instruction in bytes (including opcode)
    pub fn size(self) -> usize {
        use InstructionFormat::*;

        1 + match self.format() {
            None => 0,
            U8 | I8 | Const8 | Atom8 => 1,
            U16 | I16 => 2,
            U32 | I32 | Label | Const16 | Atom16 => 4,
        }
    }

    /// Try to convert a u8 to an opcode
    pub fn from_u8(val: u8) -> Option<Self> {
        // SAFETY: We validate that the u8 value corresponds to a valid opcode
        // The repr(u8) ensures this is a valid representation
        match val {
            0..=10 | 11..=36 | 40..=66 | 70..=85 | 90..=101 |
            110..=119 | 130..=133 | 140..=146 | 160..=170 |
            180..=188 | 200..=229 | 240..=245 | 250..=255 => unsafe {
                Some(core::mem::transmute(val))
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_names() {
        assert_eq!(Opcode::Drop.name(), "drop");
        assert_eq!(Opcode::Add.name(), "add");
        assert_eq!(Opcode::Call.name(), "call");
        assert_eq!(Opcode::Return.name(), "return");
    }

    #[test]
    fn test_opcode_format() {
        assert_eq!(Opcode::Drop.format(), InstructionFormat::None);
        assert_eq!(Opcode::PushI8.format(), InstructionFormat::I8);
        assert_eq!(Opcode::GetLoc.format(), InstructionFormat::U8);
        assert_eq!(Opcode::PushI16.format(), InstructionFormat::I16);
        assert_eq!(Opcode::GetField.format(), InstructionFormat::U16);
        assert_eq!(Opcode::PushI32.format(), InstructionFormat::I32);
        assert_eq!(Opcode::IfFalse.format(), InstructionFormat::Label);
        assert_eq!(Opcode::PushConst8.format(), InstructionFormat::Const8);
        assert_eq!(Opcode::PushConst16.format(), InstructionFormat::Const16);
    }

    #[test]
    fn test_opcode_size() {
        assert_eq!(Opcode::Drop.size(), 1);
        assert_eq!(Opcode::PushI8.size(), 2);
        assert_eq!(Opcode::GetLoc.size(), 2);
        assert_eq!(Opcode::PushI16.size(), 3);
        assert_eq!(Opcode::GetField.size(), 3);
        assert_eq!(Opcode::PushI32.size(), 5);
        assert_eq!(Opcode::IfFalse.size(), 5);
        assert_eq!(Opcode::PushConst8.size(), 2);
        assert_eq!(Opcode::PushConst16.size(), 5);
    }

    #[test]
    fn test_opcode_from_u8() {
        assert_eq!(Opcode::from_u8(0), Some(Opcode::Drop));
        assert_eq!(Opcode::from_u8(10), Some(Opcode::Undefined));
        assert_eq!(Opcode::from_u8(90), Some(Opcode::Add));
        assert_eq!(Opcode::from_u8(163), Some(Opcode::Return));
        assert_eq!(Opcode::from_u8(255), Some(Opcode::Nop));

        // Valid opcode values (new global opcodes)
        assert_eq!(Opcode::from_u8(61), Some(Opcode::GetGlobal8));
        assert_eq!(Opcode::from_u8(66), Some(Opcode::SetGlobal16));

        // PushFunc8 and PushFunc are now valid opcodes
        assert_eq!(Opcode::from_u8(33), Some(Opcode::PushFunc8));
        assert_eq!(Opcode::from_u8(34), Some(Opcode::PushFunc));

        // PushAtomString8 and PushAtomString16 are now valid opcodes
        assert_eq!(Opcode::from_u8(35), Some(Opcode::PushAtomString8));
        assert_eq!(Opcode::from_u8(36), Some(Opcode::PushAtomString16));

        // Invalid opcode values should return None (gaps in opcode numbering)
        assert_eq!(Opcode::from_u8(37), None);
        assert_eq!(Opcode::from_u8(67), None);
    }

    #[test]
    fn test_opcode_repr() {
        // Test that repr(u8) works correctly
        assert_eq!(Opcode::Drop as u8, 0);
        assert_eq!(Opcode::Add as u8, 90);
        assert_eq!(Opcode::Return as u8, 163);
        assert_eq!(Opcode::Nop as u8, 255);
    }
}
