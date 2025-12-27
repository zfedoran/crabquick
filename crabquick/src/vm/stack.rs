//! VM stack management
//!
//! This module implements the value stack and call stack for the VM.
//! The value stack holds operands for bytecode instructions, while
//! the call stack manages function call frames.

use crate::value::JSValue;
use alloc::vec::Vec;

/// Maximum stack size to prevent stack overflow
const MAX_STACK_SIZE: usize = 10000;

/// Virtual machine value stack
///
/// The value stack is used for operand evaluation during bytecode execution.
/// It grows upward and supports push/pop operations as well as indexed access.
pub struct ValueStack {
    /// Stack storage
    values: Vec<JSValue>,
    /// Maximum allowed size
    max_size: usize,
}

impl ValueStack {
    /// Creates a new value stack with the specified maximum size
    pub fn new(max_size: usize) -> Self {
        let actual_max = if max_size > MAX_STACK_SIZE {
            MAX_STACK_SIZE
        } else {
            max_size
        };

        ValueStack {
            values: Vec::with_capacity(actual_max.min(256)),
            max_size: actual_max,
        }
    }

    /// Returns the current stack size
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if the stack is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Pushes a value onto the stack
    ///
    /// Returns an error if the stack would overflow.
    #[inline]
    pub fn push(&mut self, value: JSValue) -> Result<(), StackOverflow> {
        if self.values.len() >= self.max_size {
            return Err(StackOverflow);
        }
        self.values.push(value);
        Ok(())
    }

    /// Pops a value from the stack
    ///
    /// Returns None if the stack is empty.
    #[inline]
    pub fn pop(&mut self) -> Result<JSValue, StackUnderflow> {
        self.values.pop().ok_or(StackUnderflow)
    }

    /// Peeks at the top value without removing it
    #[inline]
    pub fn peek(&self) -> Result<JSValue, StackUnderflow> {
        self.values.last().copied().ok_or(StackUnderflow)
    }

    /// Gets a value at the specified index from the bottom
    #[inline]
    pub fn get(&self, index: usize) -> Result<JSValue, StackUnderflow> {
        self.values.get(index).copied().ok_or(StackUnderflow)
    }

    /// Sets a value at the specified index from the bottom
    #[inline]
    pub fn set(&mut self, index: usize, value: JSValue) -> Result<(), StackUnderflow> {
        if index >= self.values.len() {
            return Err(StackUnderflow);
        }
        self.values[index] = value;
        Ok(())
    }

    /// Gets a value at an offset from the top (0 = top, 1 = one below, etc.)
    #[inline]
    pub fn peek_at(&self, offset: usize) -> Result<JSValue, StackUnderflow> {
        let len = self.values.len();
        if offset >= len {
            return Err(StackUnderflow);
        }
        Ok(self.values[len - 1 - offset])
    }

    /// Sets a value at an offset from the top
    #[inline]
    pub fn set_at(&mut self, offset: usize, value: JSValue) -> Result<(), StackUnderflow> {
        let len = self.values.len();
        if offset >= len {
            return Err(StackUnderflow);
        }
        self.values[len - 1 - offset] = value;
        Ok(())
    }

    /// Duplicates the top value
    #[inline]
    pub fn dup(&mut self) -> Result<(), StackError> {
        let val = self.peek()?;
        self.push(val)?;
        Ok(())
    }

    /// Swaps the top two values
    #[inline]
    pub fn swap(&mut self) -> Result<(), StackUnderflow> {
        let len = self.values.len();
        if len < 2 {
            return Err(StackUnderflow);
        }
        self.values.swap(len - 1, len - 2);
        Ok(())
    }

    /// Rotates the top N values
    ///
    /// For n=3: [a, b, c] -> [b, c, a]
    pub fn rotate(&mut self, n: usize, left: bool) -> Result<(), StackUnderflow> {
        let len = self.values.len();
        if n > len || n == 0 {
            return Err(StackUnderflow);
        }

        let start = len - n;
        if left {
            // Rotate left: first element moves to end
            let first = self.values[start];
            for i in start..len - 1 {
                self.values[i] = self.values[i + 1];
            }
            self.values[len - 1] = first;
        } else {
            // Rotate right: last element moves to start
            let last = self.values[len - 1];
            for i in (start + 1..len).rev() {
                self.values[i] = self.values[i - 1];
            }
            self.values[start] = last;
        }

        Ok(())
    }

    /// Drops N values from the top
    #[inline]
    pub fn drop_n(&mut self, n: usize) -> Result<(), StackUnderflow> {
        let len = self.values.len();
        if n > len {
            return Err(StackUnderflow);
        }
        self.values.truncate(len - n);
        Ok(())
    }

    /// Shrinks the stack to the specified size
    ///
    /// Used when returning from a function to restore the stack pointer.
    #[inline]
    pub fn truncate(&mut self, size: usize) {
        self.values.truncate(size);
    }

    /// Returns a slice of all values on the stack
    #[inline]
    pub fn as_slice(&self) -> &[JSValue] {
        &self.values
    }

    /// Clears all values from the stack
    #[inline]
    pub fn clear(&mut self) {
        self.values.clear();
    }
}

/// Stack frame for function calls
///
/// Each function call creates a new stack frame that tracks the execution state.
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Function being executed (JSValue pointing to function object)
    pub func: JSValue,
    /// Program counter within the function
    pub pc: usize,
    /// Stack pointer at function entry
    pub sp: usize,
    /// Number of arguments
    pub argc: u16,
    /// 'this' value
    pub this: JSValue,
    /// Previous exception handler PC (for try/catch)
    pub catch_offset: Option<usize>,
    /// Closure object (if this is a closure call)
    /// This HeapIndex points to a JSClosure object containing captured variable references
    pub closure: Option<crate::memory::HeapIndex>,
}

impl StackFrame {
    /// Creates a new stack frame
    pub fn new(func: JSValue, sp: usize, argc: u16, this: JSValue) -> Self {
        StackFrame {
            func,
            pc: 0,
            sp,
            argc,
            this,
            catch_offset: None,
            closure: None,
        }
    }

    /// Creates a new stack frame for a closure call
    pub fn new_closure(func: JSValue, sp: usize, argc: u16, this: JSValue, closure: crate::memory::HeapIndex) -> Self {
        StackFrame {
            func,
            pc: 0,
            sp,
            argc,
            this,
            catch_offset: None,
            closure: Some(closure),
        }
    }

    /// Sets the exception handler offset
    #[inline]
    pub fn set_catch_offset(&mut self, offset: usize) {
        self.catch_offset = Some(offset);
    }

    /// Clears the exception handler
    #[inline]
    pub fn clear_catch_offset(&mut self) {
        self.catch_offset = None;
    }
}

/// Call stack for managing function calls
pub struct CallStack {
    frames: Vec<StackFrame>,
    max_depth: usize,
}

impl CallStack {
    /// Maximum recursion depth
    const MAX_CALL_DEPTH: usize = 1000;

    /// Creates a new call stack
    pub fn new(max_depth: usize) -> Self {
        let actual_max = if max_depth > Self::MAX_CALL_DEPTH {
            Self::MAX_CALL_DEPTH
        } else {
            max_depth
        };

        CallStack {
            frames: Vec::with_capacity(actual_max.min(32)),
            max_depth: actual_max,
        }
    }

    /// Returns the current call depth
    #[inline]
    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    /// Pushes a new call frame
    #[inline]
    pub fn push(&mut self, frame: StackFrame) -> Result<(), CallStackOverflow> {
        if self.frames.len() >= self.max_depth {
            return Err(CallStackOverflow);
        }
        self.frames.push(frame);
        Ok(())
    }

    /// Pops the current call frame
    #[inline]
    pub fn pop(&mut self) -> Result<StackFrame, CallStackUnderflow> {
        self.frames.pop().ok_or(CallStackUnderflow)
    }

    /// Gets a reference to the current frame
    #[inline]
    pub fn current(&self) -> Result<&StackFrame, CallStackUnderflow> {
        self.frames.last().ok_or(CallStackUnderflow)
    }

    /// Gets a mutable reference to the current frame
    #[inline]
    pub fn current_mut(&mut self) -> Result<&mut StackFrame, CallStackUnderflow> {
        self.frames.last_mut().ok_or(CallStackUnderflow)
    }

    /// Returns true if the call stack is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Clears all frames
    #[inline]
    pub fn clear(&mut self) {
        self.frames.clear();
    }

    /// Returns a slice of all frames
    #[inline]
    pub fn frames(&self) -> &[StackFrame] {
        &self.frames
    }
}

/// Stack overflow error
#[derive(Debug, Clone, Copy)]
pub struct StackOverflow;

/// Stack underflow error
#[derive(Debug, Clone, Copy)]
pub struct StackUnderflow;

/// Call stack overflow error
#[derive(Debug, Clone, Copy)]
pub struct CallStackOverflow;

/// Call stack underflow error
#[derive(Debug, Clone, Copy)]
pub struct CallStackUnderflow;

/// Stack error (can be overflow or underflow)
#[derive(Debug)]
pub enum StackError {
    Overflow(StackOverflow),
    Underflow(StackUnderflow),
}

impl From<StackOverflow> for StackError {
    fn from(e: StackOverflow) -> Self {
        StackError::Overflow(e)
    }
}

impl From<StackUnderflow> for StackError {
    fn from(e: StackUnderflow) -> Self {
        StackError::Underflow(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_stack_push_pop() {
        let mut stack = ValueStack::new(100);

        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);

        stack.push(JSValue::from_int(1)).unwrap();
        stack.push(JSValue::from_int(2)).unwrap();
        stack.push(JSValue::from_int(3)).unwrap();

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.pop().unwrap(), JSValue::from_int(3));
        assert_eq!(stack.pop().unwrap(), JSValue::from_int(2));
        assert_eq!(stack.pop().unwrap(), JSValue::from_int(1));
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_value_stack_peek() {
        let mut stack = ValueStack::new(100);

        stack.push(JSValue::from_int(42)).unwrap();
        assert_eq!(stack.peek().unwrap(), JSValue::from_int(42));
        assert_eq!(stack.len(), 1); // Peek doesn't remove
    }

    #[test]
    fn test_value_stack_get_set() {
        let mut stack = ValueStack::new(100);

        stack.push(JSValue::from_int(1)).unwrap();
        stack.push(JSValue::from_int(2)).unwrap();
        stack.push(JSValue::from_int(3)).unwrap();

        assert_eq!(stack.get(0).unwrap(), JSValue::from_int(1));
        assert_eq!(stack.get(1).unwrap(), JSValue::from_int(2));
        assert_eq!(stack.get(2).unwrap(), JSValue::from_int(3));

        stack.set(1, JSValue::from_int(42)).unwrap();
        assert_eq!(stack.get(1).unwrap(), JSValue::from_int(42));
    }

    #[test]
    fn test_value_stack_peek_at() {
        let mut stack = ValueStack::new(100);

        stack.push(JSValue::from_int(1)).unwrap();
        stack.push(JSValue::from_int(2)).unwrap();
        stack.push(JSValue::from_int(3)).unwrap();

        assert_eq!(stack.peek_at(0).unwrap(), JSValue::from_int(3)); // Top
        assert_eq!(stack.peek_at(1).unwrap(), JSValue::from_int(2));
        assert_eq!(stack.peek_at(2).unwrap(), JSValue::from_int(1));
    }

    #[test]
    fn test_value_stack_dup() {
        let mut stack = ValueStack::new(100);

        stack.push(JSValue::from_int(42)).unwrap();
        stack.dup().unwrap();

        assert_eq!(stack.len(), 2);
        assert_eq!(stack.pop().unwrap(), JSValue::from_int(42));
        assert_eq!(stack.pop().unwrap(), JSValue::from_int(42));
    }

    #[test]
    fn test_value_stack_swap() {
        let mut stack = ValueStack::new(100);

        stack.push(JSValue::from_int(1)).unwrap();
        stack.push(JSValue::from_int(2)).unwrap();
        stack.swap().unwrap();

        assert_eq!(stack.pop().unwrap(), JSValue::from_int(1));
        assert_eq!(stack.pop().unwrap(), JSValue::from_int(2));
    }

    #[test]
    fn test_value_stack_rotate() {
        let mut stack = ValueStack::new(100);

        stack.push(JSValue::from_int(1)).unwrap();
        stack.push(JSValue::from_int(2)).unwrap();
        stack.push(JSValue::from_int(3)).unwrap();

        // Rotate left: [1, 2, 3] -> [2, 3, 1]
        stack.rotate(3, true).unwrap();
        assert_eq!(stack.pop().unwrap(), JSValue::from_int(1));
        assert_eq!(stack.pop().unwrap(), JSValue::from_int(3));
        assert_eq!(stack.pop().unwrap(), JSValue::from_int(2));
    }

    #[test]
    fn test_value_stack_overflow() {
        let mut stack = ValueStack::new(3);

        stack.push(JSValue::from_int(1)).unwrap();
        stack.push(JSValue::from_int(2)).unwrap();
        stack.push(JSValue::from_int(3)).unwrap();

        assert!(stack.push(JSValue::from_int(4)).is_err());
    }

    #[test]
    fn test_value_stack_drop_n() {
        let mut stack = ValueStack::new(100);

        stack.push(JSValue::from_int(1)).unwrap();
        stack.push(JSValue::from_int(2)).unwrap();
        stack.push(JSValue::from_int(3)).unwrap();

        stack.drop_n(2).unwrap();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.pop().unwrap(), JSValue::from_int(1));
    }

    #[test]
    fn test_call_stack() {
        let mut call_stack = CallStack::new(100);

        let frame1 = StackFrame::new(JSValue::undefined(), 0, 0, JSValue::undefined());
        call_stack.push(frame1).unwrap();

        assert_eq!(call_stack.depth(), 1);

        let frame2 = StackFrame::new(JSValue::undefined(), 10, 2, JSValue::null());
        call_stack.push(frame2).unwrap();

        assert_eq!(call_stack.depth(), 2);

        let popped = call_stack.pop().unwrap();
        assert_eq!(popped.sp, 10);
        assert_eq!(popped.argc, 2);

        assert_eq!(call_stack.depth(), 1);
    }

    #[test]
    fn test_call_stack_overflow() {
        let mut call_stack = CallStack::new(2);

        call_stack.push(StackFrame::new(JSValue::undefined(), 0, 0, JSValue::undefined())).unwrap();
        call_stack.push(StackFrame::new(JSValue::undefined(), 0, 0, JSValue::undefined())).unwrap();

        assert!(call_stack.push(StackFrame::new(JSValue::undefined(), 0, 0, JSValue::undefined())).is_err());
    }

    #[test]
    fn test_stack_frame_catch_offset() {
        let mut frame = StackFrame::new(JSValue::undefined(), 0, 0, JSValue::undefined());

        assert!(frame.catch_offset.is_none());

        frame.set_catch_offset(100);
        assert_eq!(frame.catch_offset, Some(100));

        frame.clear_catch_offset();
        assert!(frame.catch_offset.is_none());
    }
}
