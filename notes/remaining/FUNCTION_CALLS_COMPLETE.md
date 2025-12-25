# Function Calls Implementation - Complete

## Summary

All 5 phases of the function calls implementation have been completed successfully. JavaScript user-defined functions now work correctly, including recursion.

## Implementation Status: ✅ COMPLETE

### All Tests Passing
- 267 library tests pass
- 20 integration tests pass
- 6 function call tests pass:
  1. `test_function_declaration_simple` - Two-parameter function
  2. `test_function_declaration_no_params` - Zero-parameter function
  3. `test_function_one_param` - One-parameter function
  4. `test_function_with_local_var` - Function with local variable
  5. `test_function_recursive_factorial` - Recursive factorial
  6. `test_function_recursive_fibonacci` - Recursive fibonacci

## What Was Implemented

### Phase 1: Function Bytecode Compilation ✅
- **File**: `crabquick/src/compiler/codegen.rs`
- Added `FunctionBytecode` struct for compiled function metadata
- Implemented `compile_function_body()` returning (bytecode, local_count)
- Modified bytecode format to include function table:
  - Format: `[const_count][constants][atom_count][atoms][func_count][functions][code]`
  - Each function entry: `[param_count: u8][local_count: u8][bytecode_len: u32][bytecode_bytes]`

### Phase 2: Function Object Storage ✅
- **Files**: `object/function.rs`, `context.rs`, `memory/header.rs`
- Created `JSBytecodeFunction` struct with bytecode_index, param_count, local_count
- Added `Context::new_bytecode_function()` and `get_bytecode_function()` methods
- Added opcodes `PushFunc8 = 33` and `PushFunc = 34`
- Updated `from_u8()` in opcode.rs to recognize 33-34 as valid opcodes

### Phase 3: Call Frame Management ✅
- **File**: `crabquick/src/vm/interpreter.rs`
- Added `FunctionEntry` struct and `function_table: Vec<FunctionEntry>` to VM
- Enhanced `execute()` to parse function table from bytecode
- Implemented `PushFunc8` and `PushFunc` opcode handlers
- Modified `Call` opcode to detect and execute bytecode functions

### Phase 4: Local Variables ✅
- Implemented `execute_function_bytecode()` with proper base_sp handling
- `GetLoc`, `PutLoc`, `SetLoc` access stack relative to base_sp
- Local variable count now correctly computed from function body

### Phase 5: Recursion ✅
- Recursive function calls work correctly
- Tested with factorial and fibonacci

## Bugs Fixed

### Bug 1: PushFunc8/PushFunc opcodes not recognized
**Root Cause**: `Opcode::from_u8()` had range `11..=32` but PushFunc8=33, PushFunc=34
**Fix**: Changed range to `11..=34` in `bytecode/opcode.rs:751`

### Bug 2: Function bytecode missing function table header
**Root Cause**: `generate_raw()` didn't write function count header
**Fix**: Added func_count and function entries to `generate_raw()` in `codegen.rs`

### Bug 3: local_count not accounting for var declarations
**Root Cause**: local_count was set to params.len() ignoring body vars
**Fix**: `compile_function_body()` now returns actual local count from scope

### Bug 4: Test bytecode missing function table header
**Root Cause**: Unit tests used old bytecode format without func_count
**Fix**: Updated test helpers to include `0u16` for function count

## Bytecode Format

```
Main Program:
[const_count: u16]
[for each constant: type_byte + usize_value]
[atom_count: u16]
[for each atom: len_u16 + utf8_bytes]
[func_count: u16]
[for each function: param_count_u8 + local_count_u8 + bytecode_len_u32 + bytecode_bytes]
[main bytecode]

Function Bytecode (same format, nested):
[const_count: u16][constants...][atom_count: u16][atoms...][func_count: u16][funcs...][code]
```

## Design Decisions

1. **Self-contained function bytecode**: Each function has its own constant pool, atom table, and nested function table
2. **Recursive execution model**: Call creates new BytecodeReader for function bytecode
3. **Stack-relative locals**: GetLoc/PutLoc/SetLoc use base_sp offset for isolation

## Known Limitations

1. No closure support (Phase 6 - deferred)
2. No function expressions (only declarations)
3. No default/rest parameters
4. No arrow functions
5. No 'arguments' object
6. No call stack depth limit (should add 512 max)

## File Summary

| File | Changes | Purpose |
|------|---------|---------|
| `compiler/codegen.rs` | Major | Function compilation, bytecode generation |
| `bytecode/opcode.rs` | Minor | PushFunc8/PushFunc opcodes, from_u8 fix |
| `bytecode/format.rs` | Tiny | Made read_u8() public |
| `object/function.rs` | Medium | JSBytecodeFunction struct |
| `context.rs` | Medium | new_bytecode_function(), get_bytecode_function() |
| `vm/interpreter.rs` | Major | Function table parsing, bytecode execution |
| `engine.rs` | Minor | 6 new test cases |
| `tests/vm_integration.rs` | Minor | Updated bytecode helper |

## Success Criteria

| Criterion | Status |
|-----------|--------|
| `function add(a, b) { return a + b; }` compiles | ✅ PASS |
| `add(2, 3)` returns 5 | ✅ PASS |
| Local variables work | ✅ PASS |
| `factorial(5)` returns 120 | ✅ PASS |
| `fib(10)` returns 55 | ✅ PASS |

## Conclusion

JavaScript function calls are now fully working in CrabQuick. The engine can:
- Compile function declarations to bytecode
- Store function objects on the heap
- Execute function bytecode with proper local variable handling
- Support recursive function calls
- Return computed values correctly

This completes Priority 1 from the remaining work overview. The next priorities are:
- Priority 2: Type Coercion ("5" + 3, etc.)
- Priority 3: Built-in Methods (Array.push, String.slice, etc.)
