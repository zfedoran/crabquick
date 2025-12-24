//! Exception handling
//!
//! Provides exception state management and stack trace capture.

use crate::value::JSValue;
use alloc::vec::Vec;

/// Stack frame information for stack traces
#[derive(Debug, Clone)]
pub struct StackTraceFrame {
    /// Function name (if available)
    pub function_name: Option<JSValue>,
    /// Source file name (if available)
    pub file_name: Option<JSValue>,
    /// Line number (if available)
    pub line_number: Option<u32>,
    /// Program counter
    pub pc: usize,
}

impl StackTraceFrame {
    /// Creates a new stack trace frame
    pub fn new(pc: usize) -> Self {
        StackTraceFrame {
            function_name: None,
            file_name: None,
            line_number: None,
            pc,
        }
    }

    /// Sets the function name
    pub fn with_function_name(mut self, name: JSValue) -> Self {
        self.function_name = Some(name);
        self
    }

    /// Sets the file name
    pub fn with_file_name(mut self, name: JSValue) -> Self {
        self.file_name = Some(name);
        self
    }

    /// Sets the line number
    pub fn with_line_number(mut self, line: u32) -> Self {
        self.line_number = Some(line);
        self
    }
}

/// VM exception with stack trace
pub struct VMException {
    /// Exception value
    value: JSValue,
    /// Stack trace
    stack_trace: Vec<StackTraceFrame>,
}

impl VMException {
    /// Creates a new exception
    pub fn new(value: JSValue) -> Self {
        VMException {
            value,
            stack_trace: Vec::new(),
        }
    }

    /// Returns the exception value
    pub fn value(&self) -> JSValue {
        self.value
    }

    /// Returns the stack trace
    pub fn stack_trace(&self) -> &[StackTraceFrame] {
        &self.stack_trace
    }

    /// Adds a frame to the stack trace
    pub fn push_frame(&mut self, frame: StackTraceFrame) {
        self.stack_trace.push(frame);
    }

    /// Captures stack trace from the call stack
    pub fn capture_stack_trace(&mut self, frames: &[super::StackFrame]) {
        self.stack_trace.clear();

        for (i, frame) in frames.iter().enumerate().rev() {
            let trace_frame = StackTraceFrame::new(frame.pc);
            self.stack_trace.push(trace_frame);

            // Limit stack trace depth to prevent excessive memory usage
            if i >= 100 {
                break;
            }
        }
    }

    /// Clears the stack trace
    pub fn clear_stack_trace(&mut self) {
        self.stack_trace.clear();
    }

    /// Returns the stack trace depth
    pub fn stack_depth(&self) -> usize {
        self.stack_trace.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exception_new() {
        let exc = VMException::new(JSValue::undefined());
        assert_eq!(exc.value(), JSValue::undefined());
        assert_eq!(exc.stack_depth(), 0);
    }

    #[test]
    fn test_exception_push_frame() {
        let mut exc = VMException::new(JSValue::from_int(42));

        exc.push_frame(StackTraceFrame::new(0));
        exc.push_frame(StackTraceFrame::new(10));
        exc.push_frame(StackTraceFrame::new(20));

        assert_eq!(exc.stack_depth(), 3);
        assert_eq!(exc.stack_trace()[0].pc, 0);
        assert_eq!(exc.stack_trace()[1].pc, 10);
        assert_eq!(exc.stack_trace()[2].pc, 20);
    }

    #[test]
    fn test_stack_trace_frame_builder() {
        let frame = StackTraceFrame::new(100)
            .with_function_name(JSValue::from_int(1))
            .with_file_name(JSValue::from_int(2))
            .with_line_number(42);

        assert_eq!(frame.pc, 100);
        assert!(frame.function_name.is_some());
        assert!(frame.file_name.is_some());
        assert_eq!(frame.line_number, Some(42));
    }

    #[test]
    fn test_clear_stack_trace() {
        let mut exc = VMException::new(JSValue::null());
        exc.push_frame(StackTraceFrame::new(0));
        exc.push_frame(StackTraceFrame::new(10));

        assert_eq!(exc.stack_depth(), 2);

        exc.clear_stack_trace();
        assert_eq!(exc.stack_depth(), 0);
    }
}
