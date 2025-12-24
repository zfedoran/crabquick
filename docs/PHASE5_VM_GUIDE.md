# Phase 5: Virtual Machine Developer Guide

## Quick Start

### Executing Bytecode

```rust
use mquickjs::{Context, bytecode::{BytecodeWriter, Instruction, Opcode}};

// Create context
let mut ctx = Context::new(8192);

// Build bytecode
let mut writer = BytecodeWriter::new();
writer.emit(&Instruction::new(Opcode::Push2));
writer.emit(&Instruction::new(Opcode::Push3));
writer.emit(&Instruction::new(Opcode::Add));
writer.emit(&Instruction::new(Opcode::Return));

// Store in heap
let bytecode = writer.finish();
let bc_index = ctx.alloc_byte_array(bytecode.len()).unwrap();
unsafe {
    let bc_array = ctx.get_byte_array_mut(bc_index).unwrap();
    let slice = bc_array.as_full_mut_slice();
    slice[..bytecode.len()].copy_from_slice(&bytecode);
    bc_array.header_mut().set_count(bytecode.len() as u32);
}

// Execute
match ctx.execute_bytecode(bc_index) {
    Ok(result) => println!("Result: {:?}", result),
    Err(exception) => println!("Exception: {:?}", exception),
}
```

## VM Architecture

### Components

1. **ValueStack** - Operand stack for expression evaluation
2. **CallStack** - Function call frames
3. **VM** - Main interpreter with execution loop
4. **Context** - Integration point for bytecode execution

### Execution Flow

```
Context::execute_bytecode()
    -> VM::execute()
        -> VM::run_loop()
            -> VM::execute_instruction()
                -> opcode handlers
```

## Stack Operations

### ValueStack API

```rust
use mquickjs::vm::ValueStack;

let mut stack = ValueStack::new(1000);

// Push/pop
stack.push(JSValue::from_int(42)).unwrap();
let val = stack.pop().unwrap();

// Peek without removing
let top = stack.peek().unwrap();

// Indexed access
stack.set(0, JSValue::from_int(100)).unwrap();
let val = stack.get(0).unwrap();

// Stack manipulation
stack.dup().unwrap();        // Duplicate top
stack.swap().unwrap();       // Swap top two
stack.rotate(3, true).unwrap(); // Rotate top 3 left
stack.drop_n(2).unwrap();    // Drop top 2 values
```

### CallStack API

```rust
use mquickjs::vm::{CallStack, StackFrame};

let mut call_stack = CallStack::new(100);

// Push frame
let frame = StackFrame::new(
    func_val,     // JSValue
    0,            // stack pointer
    2,            // argument count
    this_val,     // this value
);
call_stack.push(frame).unwrap();

// Access current frame
let frame = call_stack.current_mut().unwrap();
frame.pc = 100;  // Update PC

// Exception handling
frame.set_catch_offset(200);  // Set exception handler

// Pop frame
let frame = call_stack.pop().unwrap();
```

## Opcode Implementation

### Adding a New Opcode

1. **Define in Opcode enum** (`bytecode/opcode.rs`):
```rust
pub enum Opcode {
    // ...
    MyNewOp = 200,
}
```

2. **Add name** (`bytecode/opcode.rs`):
```rust
impl Opcode {
    pub fn name(self) -> &'static str {
        match self {
            // ...
            Opcode::MyNewOp => "my_new_op",
        }
    }
}
```

3. **Add format** (`bytecode/opcode.rs`):
```rust
impl Opcode {
    pub fn format(self) -> InstructionFormat {
        match self {
            // ...
            Opcode::MyNewOp => InstructionFormat::U8,
        }
    }
}
```

4. **Implement handler** (`vm/interpreter.rs`):
```rust
fn execute_instruction(...) -> Result<Option<JSValue>, JSValue> {
    match instruction.opcode {
        // ...
        MyNewOp => {
            if let Operand::U8(val) = instruction.operand {
                // Implementation here
                self.value_stack.push(JSValue::from_int(val as i32))
                    .map_err(|_| self.throw_error(ctx, "Stack overflow"))?;
                Ok(None)
            } else {
                Err(self.throw_error(ctx, "Invalid operand"))
            }
        }
    }
}
```

## Error Handling

### Stack Errors

```rust
use mquickjs::vm::{StackOverflow, StackUnderflow, CallStackOverflow};

// Handle stack overflow
match stack.push(value) {
    Ok(()) => { /* success */ }
    Err(StackOverflow) => { /* handle overflow */ }
}

// Handle underflow
match stack.pop() {
    Ok(val) => { /* use val */ }
    Err(StackUnderflow) => { /* handle underflow */ }
}
```

### Exception Handling

```rust
// Throw exception
return Err(error_value);

// Catch exception
match ctx.execute_bytecode(bc_index) {
    Ok(result) => { /* normal return */ }
    Err(exception) => { /* handle exception */ }
}

// Set exception handler in frame
frame.set_catch_offset(handler_pc);
```

## Type Conversions

### To Number

```rust
// VM internal
let num = self.to_number(ctx, val)?;

// Context method
let num = ctx.get_number(val).unwrap();
```

### To Boolean

```rust
// VM internal
let bool_val = self.to_boolean(val);

// JSValue method
let bool_val = val.to_bool().unwrap();
```

### Type Checking

```rust
if val.is_int() { /* ... */ }
if val.is_null() { /* ... */ }
if val.is_undefined() { /* ... */ }
if val.is_bool() { /* ... */ }
if val.is_ptr() { /* ... */ }
```

## Operator Implementation

### Arithmetic Operators

```rust
impl VM {
    fn op_add(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        ctx.new_number(a_num + b_num).map_err(|_| JSValue::undefined())
    }
}
```

### Comparison Operators

```rust
impl VM {
    fn op_lt(&self, ctx: &Context, a: JSValue, b: JSValue) -> Result<bool, JSValue> {
        let a_num = ctx.get_number(a).or_else(|| a.to_int().map(|i| i as f64)).unwrap_or(0.0);
        let b_num = ctx.get_number(b).or_else(|| b.to_int().map(|i| i as f64)).unwrap_or(0.0);
        Ok(a_num < b_num)
    }
}
```

### Bitwise Operators

```rust
impl VM {
    fn op_bit_and(&self, ctx: &mut Context, a: JSValue, b: JSValue) -> Result<JSValue, JSValue> {
        let a_int = a.to_int().unwrap_or(0);
        let b_int = b.to_int().unwrap_or(0);
        Ok(JSValue::from_int(a_int & b_int))
    }
}
```

## Control Flow

### Conditional Jumps

```rust
// if_false
if let Operand::Label(offset) = instruction.operand {
    let cond = self.value_stack.pop()
        .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
    if !self.to_boolean(cond) {
        reader.set_pc((reader.pc() as i32 + offset) as usize);
    }
    Ok(None)
}
```

### Unconditional Jumps

```rust
// goto
if let Operand::Label(offset) = instruction.operand {
    reader.set_pc((reader.pc() as i32 + offset) as usize);
    Ok(None)
}
```

### Return

```rust
// return
let ret_val = self.value_stack.pop()
    .map_err(|_| self.throw_error(ctx, "Stack underflow"))?;
Ok(Some(ret_val))  // Signals end of execution
```

## Testing

### Unit Tests

```rust
#[test]
fn test_my_opcode() {
    let mut vm = VM::new();
    let mut ctx = Context::new(4096);

    let mut writer = BytecodeWriter::new();
    writer.emit(&Instruction::new(Opcode::MyNewOp));
    writer.emit(&Instruction::new(Opcode::Return));

    let bytecode = writer.finish();
    let bc_index = ctx.alloc_byte_array(bytecode.len()).unwrap();

    unsafe {
        let bc_array = ctx.get_byte_array_mut(bc_index).unwrap();
        let slice = bc_array.as_full_mut_slice();
        slice[..bytecode.len()].copy_from_slice(&bytecode);
        bc_array.header_mut().set_count(bytecode.len() as u32);
    }

    let result = vm.execute(&mut ctx, bc_index).unwrap();
    assert_eq!(result, expected_value);
}
```

### Integration Tests

```rust
#[test]
fn test_complex_expression() {
    let mut ctx = Context::new(4096);

    // Build bytecode for: (a + b) * c
    let instructions = vec![
        Instruction::with_i8(Opcode::PushI8, 10), // a = 10
        Instruction::with_i8(Opcode::PushI8, 5),  // b = 5
        Instruction::new(Opcode::Add),             // a + b
        Instruction::new(Opcode::Push2),          // c = 2
        Instruction::new(Opcode::Mul),             // (a + b) * c
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    let num = ctx.get_number(result).unwrap();
    assert_eq!(num, 30.0);
}
```

## Performance Tips

1. **Pre-allocate stacks**: Use appropriate initial sizes
2. **Minimize heap allocation**: Reuse VM instances when possible
3. **Inline critical paths**: Push/pop are already inlined
4. **Avoid unnecessary conversions**: Keep values in native format
5. **Use appropriate operand sizes**: I8 < I16 < I32

## Memory Management

### Stack Sizing

```rust
// Create VM with custom stack sizes
let vm = VM::with_stack_sizes(
    2000,  // value stack size
    200    // call stack depth
);
```

### Memory Limits

- Max value stack: 10,000 elements
- Max call depth: 1,000 frames
- Stack trace limit: 100 frames

### GC Integration

All values on the stack are scannable by the GC. The VM doesn't need to register roots explicitly - the Context handles this through the value stack.

## Debugging

### Stack Inspection

```rust
// Get current stack state
let stack_slice = vm.value_stack.as_slice();
for (i, val) in stack_slice.iter().enumerate() {
    println!("Stack[{}] = {:?}", i, val);
}
```

### Frame Inspection

```rust
// Get current frame
if let Ok(frame) = vm.call_stack.current() {
    println!("PC: {}", frame.pc);
    println!("SP: {}", frame.sp);
    println!("Args: {}", frame.argc);
}
```

### Exception Stack Traces

```rust
use mquickjs::vm::VMException;

let mut exc = VMException::new(error_value);
exc.capture_stack_trace(call_stack.frames());

for frame in exc.stack_trace() {
    println!("  at PC {}", frame.pc);
    if let Some(name) = frame.function_name {
        println!("    function: {:?}", name);
    }
}
```

## Common Patterns

### Simple Calculation

```rust
// push operands
// execute operation
// return result
```

### Conditional

```rust
// evaluate condition
// if_false to else/end
// then block
// goto end
// else block
// end
```

### Loop

```rust
// loop_start:
//   condition
//   if_false to loop_end
//   body
//   goto loop_start
// loop_end:
```

### Try-Catch

```rust
// push_catch_offset to catch_handler
// try block
// goto finally/end
// catch_handler:
//   catch block
// finally/end:
```

## Limitations (Current Phase)

1. **No local variables**: GetLoc/PutLoc not implemented
2. **No property access**: GetField/SetField not implemented
3. **No function calls**: Call/CallMethod not implemented
4. **No closures**: FClosure not implemented
5. **No iterators**: ForIn/ForOf not implemented

These will be added in future phases.

## See Also

- [Phase 5 Implementation Summary](../PHASE5_IMPLEMENTATION_SUMMARY.md)
- [Opcode Reference](../mquickjs/src/bytecode/opcode.rs)
- [VM Tests](../mquickjs/tests/vm_integration.rs)
- [Stack Implementation](../mquickjs/src/vm/stack.rs)
- [Interpreter Implementation](../mquickjs/src/vm/interpreter.rs)
