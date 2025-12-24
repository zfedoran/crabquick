# MicroQuickJS Documentation Index

## Quick Navigation

### For Understanding the Architecture
Start here if you want a high-level overview:
- **[README.md](README.md)** - Overview, statistics, and roadmap
- **[architecture.md](architecture.md)** - System design and philosophy

### For Implementation Details
Deep dives into specific subsystems:
- **[data-structures.md](data-structures.md)** - All core data types (2,738 lines total docs)
- **[memory-management.md](memory-management.md)** - GC and allocation strategies
- **[execution-model.md](execution-model.md)** - Parser, compiler, and VM
- **[modules.md](modules.md)** - File-by-file breakdown

## Reading Order by Goal

### Goal: Port to Rust
1. Start: [README.md](README.md) - Get overview and roadmap
2. Study: [data-structures.md](data-structures.md) - Understand core types
3. Deep dive: [memory-management.md](memory-management.md) - Critical for safe Rust
4. Learn: [execution-model.md](execution-model.md) - VM implementation
5. Reference: [modules.md](modules.md) - Map C code to modules

### Goal: Understand MicroQuickJS
1. [architecture.md](architecture.md) - Design philosophy
2. [execution-model.md](execution-model.md) - How JS runs
3. [data-structures.md](data-structures.md) - Internal representations
4. [memory-management.md](memory-management.md) - Memory model

### Goal: Extend MicroQuickJS
1. [modules.md](modules.md) - Find relevant code
2. [data-structures.md](data-structures.md) - Understand data layout
3. [memory-management.md](memory-management.md) - GC-safe coding
4. [execution-model.md](execution-model.md) - Add opcodes/features

## Key Sections by Topic

### Memory and GC
- [memory-management.md](memory-management.md) - Complete GC algorithm
- [data-structures.md](data-structures.md#jsgcref-mquickjsh) - GC reference types
- [architecture.md](architecture.md#memory-layout) - Memory layout

### Values and Objects
- [data-structures.md](data-structures.md#jsvalue-mquickjsh) - Value encoding
- [data-structures.md](data-structures.md#jsobject-mquickjsc) - Object structure
- [data-structures.md](data-structures.md#property-system) - Properties

### Bytecode and VM
- [execution-model.md](execution-model.md#bytecode-format) - Instruction encoding
- [execution-model.md](execution-model.md#virtual-machine) - VM loop
- [modules.md](modules.md#6-bytecode-interpreter-lines-4700-7200) - Opcode implementations

### Parser and Compiler
- [execution-model.md](execution-model.md#parsing-and-compilation) - Compilation pipeline
- [execution-model.md](execution-model.md#parser-architecture) - Parser design
- [modules.md](modules.md#6-parsercompiler-lines-7200-11700) - Parser code

### Built-in Functions
- [modules.md](modules.md#7-built-in-functions-scattered-throughout) - All built-ins
- [modules.md](modules.md#supporting-files) - dtoa, libm, etc.

## Code Mapping

### C Source → Documentation
- `mquickjs.c` lines 500-600 → [modules.md](modules.md#1-memory-management)
- `mquickjs.c` lines 970-2300 → [modules.md](modules.md#2-value-operations)
- `mquickjs.c` lines 2400-3300 → [modules.md](modules.md#3-property-system)
- `mquickjs.c` lines 4700-7200 → [modules.md](modules.md#5-bytecode-interpreter)
- `mquickjs.c` lines 11780-12700 → [memory-management.md](memory-management.md#garbage-collection)

### Structures → Documentation
- `JSValue` → [data-structures.md](data-structures.md#jsvalue-mquickjsh)
- `JSObject` → [data-structures.md](data-structures.md#jsobject-mquickjsc)
- `JSContext` → [data-structures.md](data-structures.md#jscontext-mquickjsc)
- `JSFunctionBytecode` → [data-structures.md](data-structures.md#jsfunctionbytecode-mquickjsc)

## Statistics

- **Total documentation**: ~2,700 lines across 6 files
- **C source analyzed**: ~18,000 lines (mquickjs.c)
- **Total project**: ~28,500 lines of C code
- **Coverage**: All major subsystems documented

## Critical Sections for Rust Port

### Must Read
1. [memory-management.md](memory-management.md#gc-safe-programming) - GC safety
2. [data-structures.md](data-structures.md#important-patterns-for-rust-port) - Rust patterns
3. [execution-model.md](execution-model.md#rust-port-considerations) - VM considerations

### Essential Patterns
- Tagged pointers: [data-structures.md](data-structures.md#tagged-pointers)
- Bit fields: [data-structures.md](data-structures.md#bit-fields)
- Unions: [data-structures.md](data-structures.md#union-discriminants)
- Pointer stability: [memory-management.md](memory-management.md#the-pointer-invalidation-problem)

### Safety Invariants
- GC rooting: [memory-management.md](memory-management.md#solution-jsgcref)
- Stack management: [execution-model.md](execution-model.md#call-frames)
- Bounds checking: Throughout all documents

## Quick Reference

### Value Tags
See [data-structures.md](data-structures.md#value-encoding)

### Opcodes
See [modules.md](modules.md#opcode-categories)

### Memory Layout
See [architecture.md](architecture.md#memory-layout)

### Call Convention
See [execution-model.md](execution-model.md#call-frames)

### GC Algorithm
See [memory-management.md](memory-management.md#phase-2-sweep-and-compact)

---

**Total Documentation**: 67 KB across 6 markdown files
**Lines of Documentation**: 2,738 lines
**Source Coverage**: Complete analysis of all major systems
