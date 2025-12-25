# MicroQuickJS Architecture Overview

## Project Goal
MicroQuickJS (MQuickJS) is a JavaScript engine designed for embedded systems, capable of running JavaScript programs using as little as 10 kB of RAM with the entire engine requiring about 100 kB of ROM (ARM Thumb-2 code).

## High-Level Design Philosophy

### Memory Constraints
- Designed to run on systems with extremely limited RAM (10 kB minimum)
- No dependency on standard C library malloc/free/printf
- Custom memory allocator using a user-provided fixed buffer
- All memory comes from a single contiguous buffer provided at context creation

### Key Architectural Differences from QuickJS
1. **Garbage Collection**: Uses tracing + compacting GC instead of reference counting
2. **Value Representation**: CPU-word-sized values (32-bit on 32-bit systems)
3. **String Encoding**: UTF-8 internally (vs 8/16-bit arrays in QuickJS)
4. **VM Stack**: Does not use CPU stack for execution
5. **Standard Library**: Compiled to ROM-resident C structures

### JavaScript Subset ("Stricter Mode")
- ES5 compliance with select ES6+ features
- Strict mode only (no `with`, global vars must be declared)
- No array holes
- No direct `eval` (only global eval)
- No value boxing (no `new Number(1)`)
- Simplified regexp and Date support

## System Architecture

### Memory Layout
```
┌─────────────────────┐ ← mem_start (user-provided buffer)
│   Heap Base         │
│   (allocated up)    │
│                     │
├─────────────────────┤ ← heap_free (grows upward)
│   Free Area         │
│                     │
├─────────────────────┤ ← stack_bottom (min safe position)
│   Stack             │
│   (grows down)      │
├─────────────────────┤ ← sp (current stack pointer)
│                     │
├─────────────────────┤ ← stack_top
│   JSContext         │
└─────────────────────┘ ← mem_start + mem_size
```

### Core Components

#### 1. Memory Management Subsystem
- **Custom Allocator**: Simple bump allocator (heap_free pointer)
- **GC**: Two-phase mark-and-compact with threading (Deutsch-Schorr-Waite-like)
- **No Fragmentation**: Compaction moves all live objects together
- **ROM Support**: Can reference read-only data structures

#### 2. Value System
- **JSValue**: Single word (32 or 64 bits)
  - 31-bit integers (1-bit tag)
  - Single unicode codepoint (short strings)
  - 64-bit float with small exponent (on 64-bit systems)
  - Pointer to heap-allocated block

#### 3. Parser/Compiler
- **Single-pass compilation**: No AST, generates bytecode directly
- **Non-recursive**: Bounded C stack usage via state machine
- **Bytecode optimization**: Multiple tricks during single-pass generation
- **Debug info compression**: Exponential-Golomb encoding for line/column numbers

#### 4. Virtual Machine
- **Stack-based bytecode**: Similar to QuickJS
- **Custom VM stack**: Independent of C stack
- **Indirect atom references**: Through atom table
- **Fast paths**: Optimized for common operations (integer math, array access)

#### 5. Object System
- **Minimum 12 bytes**: For basic object on 32-bit system
- **Hash table properties**: Dynamic property storage
- **Prototype chain**: Standard JavaScript prototype-based inheritance
- **Class system**: Support for built-in and user classes

## File Organization

### Core Engine Files
- `mquickjs.h` - Public API header
- `mquickjs.c` - Main engine implementation (~18K LOC)
- `mquickjs_priv.h` - Private definitions and function declarations
- `mquickjs_opcode.h` - Bytecode opcode definitions

### Supporting Libraries
- `dtoa.c/h` - Double to ASCII conversion
- `libm.c/h` - Tiny math library with optional soft-float
- `cutils.c/h` - C utility functions
- `list.h` - Intrusive linked list macros

### Build System
- `mquickjs_build.c/h` - Tool to compile standard library to C structures
- `mqjs.c` - REPL and command-line interface
- `mqjs_stdlib.c` - Standard library definition

### Testing
- `example.c` - C API usage example
- `tests/*.js` - Test suite

## Execution Flow

### Context Creation
1. User provides memory buffer
2. `JS_NewContext()` initializes JSContext at end of buffer
3. Standard library instantiated (minimal RAM, mostly ROM references)
4. Heap and stack regions established

### Script Execution
1. **Parse**: `JS_Parse()` → bytecode function
2. **Create Closure**: Wrap bytecode in closure object
3. **Execute**: `JS_Run()` → `JS_Call()` → bytecode interpreter
4. **Return**: Result value returned, GC can collect temporaries

### Garbage Collection Trigger Points
- When allocation fails (heap_free meets stack_bottom)
- Explicit `JS_GC()` call
- Before critical operations requiring guaranteed memory

## Design Patterns for Rust Port

### Challenges to Address

1. **Manual Memory Management**
   - C code uses raw pointer arithmetic extensively
   - GC threading modifies pointers in-place
   - Rust needs safe abstractions while maintaining efficiency

2. **Union Types**
   - JSObject uses C unions for different object types
   - Consider Rust enums with custom layout control

3. **Pointer Stability**
   - GC moves objects, requires special handling
   - JSGCRef provides stable references during allocations
   - Rust version needs similar mechanism (indices? handles?)

4. **C-specific Optimizations**
   - Bit packing in structs
   - Computed gotos in VM loop
   - Unaligned access patterns

5. **No Global State**
   - Everything scoped to JSContext
   - Good for Rust's ownership model

### Opportunities

1. **Type Safety**
   - Replace void* with typed references
   - Use enums for tagged values
   - Stronger guarantees about object lifetimes

2. **Memory Safety**
   - Eliminate buffer overruns
   - Bounds checking on arrays
   - Safe pointer dereferencing

3. **Error Handling**
   - Replace error codes with Result types
   - Better exception handling

4. **Modern Features**
   - Use Rust's pattern matching
   - Iterator traits for loops
   - Type state pattern for parser

## Performance Characteristics

### Time Complexity
- **Property lookup**: O(1) average (hash table)
- **GC mark**: O(live objects)
- **GC compact**: O(heap size)
- **Array access**: O(1) for integer indices

### Space Complexity
- **Minimum object**: 12 bytes (32-bit) or 24 bytes (64-bit)
- **Property**: 12 bytes (32-bit) minimum
- **String**: Header + UTF-8 bytes
- **GC overhead**: Few bits per allocated block

## Build and Deployment

### Compilation Process
1. Build standard library compiler (`mquickjs_build`)
2. Compile JS standard library to C structures
3. Build main engine with embedded stdlib
4. Link application code

### Target Platforms
- Embedded ARM (Thumb-2)
- x86/x86_64
- Cross-compilation support
- Optional soft-float for systems without FPU

### ROM vs RAM Trade-offs
- Standard library in ROM (fast instantiation, no RAM cost)
- Bytecode can be flashed to ROM (with relocation)
- User chooses RAM budget at context creation
