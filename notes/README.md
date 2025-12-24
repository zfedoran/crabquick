# MicroQuickJS Analysis Notes

This directory contains comprehensive documentation of the MicroQuickJS (mquickjs) project from https://github.com/bellard/mquickjs.git, prepared for creating a Rust port.

## Documents

### [architecture.md](architecture.md)
Overall design and structure of MicroQuickJS:
- Design philosophy and constraints
- Memory layout strategy
- Core components overview
- Key differences from QuickJS
- System architecture diagrams
- File organization
- Build and deployment process

**Key Insights:**
- 10 kB minimum RAM requirement
- Custom memory allocator with tracing/compacting GC
- No standard C library dependencies
- Single contiguous memory buffer for all allocations
- ROM-based standard library for instant instantiation

### [data-structures.md](data-structures.md)
Detailed documentation of all key data structures:
- JSValue encoding (NaN-boxing-like)
- Memory block headers and tags
- Object representation
- Property storage (hash tables)
- String storage (UTF-8)
- Function bytecode structures
- Context and parser state
- Stack frame layout

**Key Insights:**
- Pointer tagging for efficient value representation
- Bit-packed structures to minimize memory
- Union types for different object classes
- Flexible array members for variable-size data
- GC reference system for pointer stability

### [modules.md](modules.md)
Breakdown of each module/component:
- File-by-file description of ~18K lines of C code
- Function categories and responsibilities
- Opcode definitions and categories
- Built-in functions by category
- Supporting libraries (dtoa, libm, cutils)
- Build system and code generation
- Module dependencies

**Key Insights:**
- Monolithic design in single .c file
- ~260 bytecode opcodes with optimized variants
- Extensive built-in library in C
- Build-time tools for stdlib compilation
- Clean separation of parser/VM/runtime

### [memory-management.md](memory-management.md)
In-depth analysis of memory management:
- Bump allocator strategy
- Garbage collection algorithm (mark & compact)
- Pointer threading for compaction
- Weak references and finalizers
- GC-safe programming patterns
- JSGCRef system for temporary roots
- ROM support for read-only data
- Debug modes and testing strategies

**Key Insights:**
- O(1) allocation with bump pointer
- Compacting GC eliminates fragmentation
- Deutsch-Schorr-Waite pointer threading
- Explicit GC root management required
- DEBUG_GC mode for catching pointer bugs

### [execution-model.md](execution-model.md)
How JavaScript code is parsed and executed:
- Parser architecture (non-recursive)
- Lexical analysis and tokenization
- One-pass compilation to bytecode
- Bytecode format and encoding
- Stack-based VM with computed goto
- Function calling conventions
- Exception handling mechanism
- Tail call optimization
- Fast paths for common operations

**Key Insights:**
- Non-recursive parser for bounded stack
- Direct bytecode emission without AST
- Computed goto for fast dispatch
- Multiple optimization techniques (peephole, dead code elimination)
- Interrupt polling for user control

## Statistics

### Codebase Size
- **mquickjs.c**: ~18,000 lines (main engine)
- **Total C code**: ~28,500 lines
- **Header files**: ~2,500 lines
- **Supporting libraries**: ~4,400 lines (dtoa, libm, cutils)
- **Build tools**: ~1,000 lines

### Complexity Metrics
- **Opcodes**: ~170 primary + ~90 optimized variants = 260 total
- **Built-in classes**: ~25
- **Built-in functions**: ~200+
- **Memory tags**: 8 types
- **Value tags**: 7 special types + int/ptr/float

### Memory Characteristics
- **Minimum RAM**: 10 KB
- **Typical ROM**: 100 KB (ARM Thumb-2)
- **Object overhead**: 12 bytes (32-bit) or 24 bytes (64-bit)
- **Property overhead**: 12 bytes minimum
- **GC pause**: ~1ms per 64KB heap (typical)

## Rust Port Roadmap

### Phase 1: Core Infrastructure
1. **Memory Management**
   - Arena allocator
   - GC implementation (mark & compact)
   - Handle system for pointer stability
   - Safe abstractions over unsafe operations

2. **Value System**
   - JSValue representation
   - Type checking and conversion
   - String handling (UTF-8)
   - Numeric operations

### Phase 2: Object System
3. **Objects and Properties**
   - Object allocation and structure
   - Property hash tables
   - Prototype chains
   - Array handling

4. **Functions**
   - Closure representation
   - C function bindings
   - Call frame management

### Phase 3: Execution Engine
5. **Bytecode**
   - Opcode definitions
   - Instruction encoding/decoding
   - Constant pool management

6. **VM**
   - Bytecode interpreter
   - Stack management
   - Exception handling
   - Fast paths

### Phase 4: Compiler
7. **Lexer**
   - Tokenization
   - UTF-8 handling
   - Number/string parsing

8. **Parser**
   - Expression parsing
   - Statement parsing
   - Code generation

### Phase 5: Runtime
9. **Built-ins**
   - Object methods
   - Array methods
   - String methods
   - Math functions
   - Error handling

10. **Standard Library**
    - JSON
    - RegExp
    - Typed Arrays
    - Date (minimal)

## Critical C Patterns for Rust Translation

### 1. Tagged Pointers
**C Pattern:**
```c
#define JS_VALUE_TO_PTR(v) ((void *)((v) - 1))
#define JS_VALUE_FROM_PTR(p) ((JSWord)((p) + 1))
```

**Rust Approach:**
- Use newtype wrappers
- Implement safe accessors
- Consider NonNull<T> with PhantomData

### 2. Bit Fields
**C Pattern:**
```c
struct {
    uint32_t gc_mark: 1;
    uint32_t mtag: 3;
    uint32_t class_id: 8;
};
```

**Rust Approach:**
- Manual bit manipulation
- Use bitfield crates
- Document bit layouts clearly

### 3. Flexible Array Members
**C Pattern:**
```c
struct {
    int size;
    JSValue arr[];  // Variable length
};
```

**Rust Approach:**
- Separate allocation strategy
- Use DSTs with care
- Or split header and data

### 4. Union Discrimination
**C Pattern:**
```c
union {
    JSClosureData closure;
    JSArrayData array;
} u;
// Discriminated by class_id field
```

**Rust Approach:**
- Use enum with repr(C)
- Manual discrimination if needed
- Document safety invariants

### 5. Interior Mutability
**C Pattern:**
```c
void modify(const JSObject *obj) {
    ((JSObject *)obj)->field = value;  // Cast away const
}
```

**Rust Approach:**
- UnsafeCell where necessary
- RefCell for dynamic checks
- Document mutation points

### 6. Pointer Invalidation
**C Pattern:**
```c
JSObject *obj = get_ptr(val);
allocate_something();  // obj is now invalid!
obj->field = ...;      // Bug!
```

**Rust Approach:**
- Use handles/indices instead of pointers
- Lifetime tracking
- Arena-based allocation

## Testing Strategy

### Unit Tests
- Value encoding/decoding
- Memory allocation/GC
- Property lookups
- Type conversions
- Each opcode independently

### Integration Tests
- Parse and execute JS scripts
- Built-in function behavior
- Error handling
- Memory limits

### Compliance Tests
- ECMAScript conformance
- Edge cases
- Error conditions

### Performance Tests
- Benchmark suite
- Memory usage profiling
- GC pause measurement

### Fuzzing
- Parser fuzzing
- VM fuzzing
- Memory allocator fuzzing

## Resources

### Original Repository
- https://github.com/bellard/mquickjs

### Related Projects
- QuickJS: https://bellard.org/quickjs/
- Comparison with other embedded JS engines

### Reference Documentation
- ECMAScript 5.1 Specification
- JavaScript reference materials

## Next Steps

1. **Set up Rust project structure**
   - Cargo workspace
   - Module organization
   - CI/CD pipeline

2. **Implement core memory management**
   - Arena allocator
   - Basic GC (mark only first)
   - Handle system

3. **Create value system**
   - JSValue representation
   - Type checking
   - Basic conversions

4. **Prototype simple VM**
   - A few opcodes
   - Stack operations
   - Simple execution

5. **Iterate and expand**
   - Add more features incrementally
   - Test continuously
   - Benchmark regularly

## Notes for Developers

### Safety Considerations
- All unsafe code must be documented with safety invariants
- GC must maintain pointer validity
- Stack operations must be bounds-checked
- No data races (single-threaded first)

### Performance Goals
- Match or exceed C performance
- Minimize allocations
- Cache-friendly data structures
- SIMD where applicable

### API Design
- Safe by default
- Zero-cost abstractions
- Ergonomic for embedders
- Clear error messages

### Documentation
- Rustdoc for all public APIs
- Architecture decision records
- Performance characteristics
- Safety invariants

---

*Last Updated: 2025-12-24*
*Based on MicroQuickJS commit: latest as of analysis date*
