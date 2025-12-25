# Function Calls Implementation Plan

## Overview

This document outlines the phased approach to implement complete JavaScript function calls in CrabQuick. Currently, only native (C-style) function calls work. We need to implement bytecode function compilation, storage, and execution.

## Current State

### What Works
- Native function calls via `MemTag::CFunctionData` (e.g., `Math.abs(x)`)
- `Call` opcode dispatches to `ctx.call_function()`
- Function arguments are correctly popped from stack

### What's Missing
1. **Function bytecode compilation** - Functions are parsed but body isn't compiled
2. **Function object storage** - No `JSBytecodeFunction` allocation
3. **Call frame management** - `push_frame`/`pop_frame` are stubs
4. **Closure support** - Variable capture not implemented
5. **Return handling** - `Return` opcode doesn't restore frames

## Implementation Phases

---

## Phase 1: Function Bytecode Compilation

**Goal**: Compile function declarations/expressions to bytecode stored in constants.

### Files to Modify
- `crabquick/src/compiler/codegen.rs`

### Tasks

1. **Add function bytecode structure to constants**
   ```rust
   enum ConstantValue {
       Number(f64),
       String(String),
       Function { bytecode: Vec<u8>, param_count: u8, name: Option<String> },
   }
   ```

2. **Implement `compile_function_body()`**
   - Create new CodeGenerator for function scope
   - Track parameter names as local variables (slots 0..n)
   - Compile body statements
   - Emit implicit `Return` at end if no explicit return

3. **Handle `FunctionDecl` in `compile_stmt()`**
   - Compile function body to bytecode
   - Store as constant with `PushConst` index
   - Emit `DefineFunc` or store in local/global slot

4. **Handle `FunctionExpr` in `compile_expr()`**
   - Same as above but result goes on stack

### Test Cases
```javascript
function add(a, b) { return a + b; }
var mul = function(x, y) { return x * y; };
```

---

## Phase 2: Function Object Storage

**Goal**: Allocate and store bytecode functions on the heap.

### Files to Modify
- `crabquick/src/object/function.rs`
- `crabquick/src/memory/header.rs`
- `crabquick/src/context.rs`

### Tasks

1. **Define `JSBytecodeFunction` structure**
   ```rust
   #[repr(C)]
   pub struct JSBytecodeFunction {
       bytecode_index: HeapIndex,  // Points to JSByteArray
       param_count: u8,
       local_count: u8,
       flags: u8,
       name_atom: u32,  // 0 if anonymous
   }
   ```

2. **Add `MemTag::BytecodeFunction`** (if not already present)

3. **Add `Context::new_bytecode_function()`**
   ```rust
   pub fn new_bytecode_function(
       &mut self,
       bytecode: &[u8],
       param_count: u8,
       local_count: u8,
   ) -> Result<JSValue, OutOfMemory>
   ```

4. **Implement `DefineFunc` opcode** (or reuse existing mechanism)
   - Pops bytecode constant index
   - Creates function object
   - Stores in appropriate scope

### Test Cases
```javascript
function foo() { return 42; }
typeof foo  // should be "function"
```

---

## Phase 3: Call Frame Management

**Goal**: Proper call stack with frame pointers for nested calls.

### Files to Modify
- `crabquick/src/vm/call.rs`
- `crabquick/src/vm/stack.rs`
- `crabquick/src/vm/interpreter.rs`

### Tasks

1. **Enhance `StackFrame` structure**
   ```rust
   pub struct StackFrame {
       pub return_pc: usize,           // Where to continue after return
       pub return_bytecode: HeapIndex, // Which bytecode to return to
       pub base_sp: usize,             // Stack pointer at call time
       pub this: JSValue,
       pub func: JSValue,
       pub local_count: u8,            // Number of local variable slots
   }
   ```

2. **Implement `push_frame()` in call.rs**
   - Save current PC and bytecode index
   - Reserve stack space for locals
   - Initialize params from arguments

3. **Implement `pop_frame()`**
   - Restore previous PC and bytecode
   - Clean up local slots
   - Return value handling

4. **Update `Call` opcode handler**
   - Detect bytecode function (check `MemTag::BytecodeFunction`)
   - Push new frame
   - Switch execution to function bytecode
   - Don't push result yet (Return will do that)

5. **Update `Return` opcode handler**
   - Pop return value
   - Pop frame
   - Restore execution context
   - Push return value to caller's stack

### Test Cases
```javascript
function double(x) { return x * 2; }
double(21)  // should be 42
```

---

## Phase 4: Local Variables in Functions

**Goal**: Function parameters and local variables work correctly.

### Files to Modify
- `crabquick/src/compiler/codegen.rs`
- `crabquick/src/vm/interpreter.rs`
- `crabquick/src/bytecode/opcodes.rs`

### Tasks

1. **Track local variable scope in compiler**
   ```rust
   struct FunctionScope {
       locals: Vec<String>,  // param names first, then var declarations
       parent: Option<Box<FunctionScope>>,
   }
   ```

2. **Emit `GetLocal`/`SetLocal` for function-scoped variables**
   - Slot 0..param_count = parameters
   - Slot param_count.. = local vars

3. **Implement `GetLocal8`/`SetLocal8` opcodes**
   - Access stack relative to frame base pointer
   - `stack[frame.base_sp + slot_index]`

4. **Handle `var` declarations in function body**
   - Allocate slot at compile time
   - Initialize to undefined at function entry

### Test Cases
```javascript
function sum(a, b) {
    var result = a + b;
    return result;
}
sum(10, 20)  // should be 30
```

---

## Phase 5: Recursion and Nested Calls

**Goal**: Functions can call other functions and themselves.

### Files to Modify
- `crabquick/src/vm/interpreter.rs`
- `crabquick/src/vm/stack.rs`

### Tasks

1. **Verify call stack depth handling**
   - Check max call depth (e.g., 512 frames)
   - Throw "Maximum call stack exceeded" on overflow

2. **Test mutual recursion**
   ```javascript
   function isEven(n) { return n === 0 ? true : isOdd(n - 1); }
   function isOdd(n) { return n === 0 ? false : isEven(n - 1); }
   ```

3. **Test deep recursion**
   ```javascript
   function fib(n) {
       if (n <= 1) return n;
       return fib(n - 1) + fib(n - 2);
   }
   fib(10)  // should be 55
   ```

4. **Benchmark against C version**
   - Run fib(25) or fib(30)
   - Compare execution time

### Test Cases
```javascript
function factorial(n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
factorial(10)  // should be 3628800
```

---

## Phase 6: Closures (Optional/Future)

**Goal**: Functions capture variables from enclosing scope.

### Tasks (Deferred)
1. Implement `VarRef` objects for captured variables
2. Track free variables during compilation
3. Create closure object with environment pointer
4. Access captured vars via `GetVarRef`/`SetVarRef` opcodes

---

## Testing Strategy

### Unit Tests (per phase)
- Add tests to `crabquick/src/engine.rs` in `mod tests`
- Test via `engine.eval("...")` and check result

### Integration Tests
- Add to `crabquick/tests/` directory
- Full programs like fibonacci, factorial

### Benchmark
After Phase 5:
```rust
#[bench]
fn bench_fibonacci() {
    let mut engine = Engine::new(65536);
    engine.eval("function fib(n) { return n <= 1 ? n : fib(n-1) + fib(n-2); } fib(25)");
}
```

---

## Success Criteria

1. **Phase 1-2 Complete**: `function foo() { return 42; }` compiles without error
2. **Phase 3 Complete**: `foo()` executes and returns 42
3. **Phase 4 Complete**: `function add(a,b) { return a+b; } add(1,2)` returns 3
4. **Phase 5 Complete**: `fib(20)` returns 6765 in reasonable time
5. **Benchmark**: Within 2x of C MicroQuickJS performance

---

## File Summary

| Phase | Files Modified |
|-------|----------------|
| 1 | `compiler/codegen.rs` |
| 2 | `object/function.rs`, `memory/header.rs`, `context.rs` |
| 3 | `vm/call.rs`, `vm/stack.rs`, `vm/interpreter.rs` |
| 4 | `compiler/codegen.rs`, `vm/interpreter.rs` |
| 5 | `vm/interpreter.rs`, `vm/stack.rs` |
