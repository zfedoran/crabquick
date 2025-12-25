//! Function call handling

use crate::value::JSValue;

/// Call frame layout constants
pub mod frame {
    /// Offset of previous frame pointer
    pub const OFFSET_PREV_FP: isize = 0;
    /// Offset of call flags
    pub const OFFSET_CALL_FLAGS: isize = 1;
    /// Offset of current PC
    pub const OFFSET_CUR_PC: isize = 2;
    /// Offset of 'this' value
    pub const OFFSET_THIS: isize = 3;
    /// Offset of function object
    pub const OFFSET_FUNC_OBJ: isize = 4;
    /// Offset of first argument
    pub const OFFSET_ARG0: isize = 5;
}

/// Pushes a new call frame
pub fn push_frame(_argc: u16) {
    // TODO: Save previous FP
    // TODO: Set up new frame
    // TODO: Adjust SP
}

/// Pops the current call frame
pub fn pop_frame() -> JSValue {
    // TODO: Restore previous FP
    // TODO: Adjust SP
    // TODO: Return value
    JSValue::undefined()
}
