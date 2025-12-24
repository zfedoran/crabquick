# MicroQuickJS Rust Port - Implementation Plan

## Executive Summary

This document outlines a comprehensive plan for porting MicroQuickJS from C to idiomatic Rust. The goal is to create a native Rust implementation that maintains the minimal resource footprint (10 kB RAM minimum) while leveraging Rust's safety guarantees where possible.

**Key Metrics:**
- Source to port: ~18,000 lines of C (mquickjs.c) + ~4,400 lines of supporting libraries
- Target: Native Rust (not FFI bindings)
- Estimated effort: 6-9 months for full implementation
- Complexity: High (custom GC, pointer threading, non-recursive parser, bytecode VM)

## 1. Project Structure

### 1.1 Cargo Workspace Layout

```
rustmicroquickjs/
├── Cargo.toml                 # Workspace root
├── mquickjs/                  # Core engine library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs             # Public API
│       ├── context.rs         # JSContext
│       ├── value.rs           # JSValue encoding/decoding
│       ├── memory/
│       │   ├── mod.rs
│       │   ├── allocator.rs   # Bump allocator
│       │   ├── gc.rs          # Mark & compact GC
│       │   ├── handle.rs      # GC root handles
│       │   └── header.rs      # Memory block headers
│       ├── object/
│       │   ├── mod.rs
│       │   ├── object.rs      # JSObject
│       │   ├── property.rs    # Property hash tables
│       │   ├── array.rs       # Array objects
│       │   ├── function.rs    # Function/closure objects
│       │   └── string.rs      # String objects
│       ├── bytecode/
│       │   ├── mod.rs
│       │   ├── opcode.rs      # Opcode definitions
│       │   ├── format.rs      # Instruction encoding
│       │   └── constants.rs   # Constant pool
│       ├── compiler/
│       │   ├── mod.rs
│       │   ├── lexer.rs       # Tokenization
│       │   ├── parser.rs      # Parsing
│       │   ├── codegen.rs     # Bytecode generation
│       │   └── debug.rs       # Debug info (pc2line)
│       ├── vm/
│       │   ├── mod.rs
│       │   ├── interpreter.rs # Bytecode interpreter
│       │   ├── call.rs        # Function calls
│       │   ├── exception.rs   # Exception handling
│       │   └── stack.rs       # Stack management
│       ├── builtins/
│       │   ├── mod.rs
│       │   ├── object.rs      # Object methods
│       │   ├── array.rs       # Array methods
│       │   ├── string.rs      # String methods
│       │   ├── number.rs      # Number methods
│       │   ├── function.rs    # Function methods
│       │   ├── math.rs        # Math functions
│       │   ├── json.rs        # JSON support
│       │   ├── regexp.rs      # RegExp (basic)
│       │   ├── error.rs       # Error objects
│       │   └── typed_array.rs # Typed arrays
│       ├── runtime/
│       │   ├── mod.rs
│       │   ├── conversion.rs  # Type conversions
│       │   ├── operators.rs   # Operator implementations
│       │   └── compare.rs     # Comparison operations
│       └── util/
│           ├── mod.rs
│           ├── dtoa.rs        # Number formatting
│           ├── strtod.rs      # String to number
│           ├── utf8.rs        # UTF-8 utilities
│           └── bitpack.rs     # Bit manipulation helpers
├── mquickjs-build/            # Build-time stdlib compiler
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── emit.rs            # C code generation
├── mqjs/                      # CLI REPL
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── repl.rs
│       └── readline.rs
└── tests/
    ├── unit/                  # Unit tests
    ├── integration/           # Integration tests
    ├── compliance/            # ECMAScript compliance
    └── fuzz/                  # Fuzzing targets
```

### 1.2 Crate Dependencies

**Core Library (`mquickjs`):**
```toml
[dependencies]
# Consider NO dependencies for core to maintain minimal footprint
# All functionality implemented natively

[dev-dependencies]
proptest = "1.0"      # Property-based testing
criterion = "0.5"     # Benchmarking
```

**Build Tool (`mquickjs-build`):**
```toml
[dependencies]
mquickjs = { path = "../mquickjs" }
```

**CLI (`mqjs`):**
```toml
[dependencies]
mquickjs = { path = "../mquickjs" }
rustyline = "13.0"    # Readline support (optional feature)
```

**Rationale for Minimal Dependencies:**
- MicroQuickJS's value proposition is minimal footprint
- Dependencies add bloat and attack surface
- Native implementations ensure predictable behavior
- Easier to target embedded/no_std environments

**Potential Future Dependencies (if justified):**
- `hashbrown` - For property hash tables (only if demonstrably better)
- `bumpalo` - For arena allocation patterns (reference only, likely custom impl)

## 2. Implementation Phases

### Phase 0: Foundation (Weeks 1-2)

**Goal:** Set up project infrastructure and core abstractions.

**Tasks:**
- [ ] Create Cargo workspace
- [ ] Set up CI/CD pipeline (GitHub Actions)
- [ ] Configure rustfmt, clippy
- [ ] Create basic test harness
- [ ] Define public API surface in `lib.rs`
- [ ] Write architecture decision records (ADRs)

**Deliverables:**
- Compiling skeleton project
- CI running tests
- Initial documentation

### Phase 1: Memory Management (Weeks 3-6)

**Goal:** Implement custom allocator and garbage collector.

**Tasks:**
- [ ] Implement memory block headers with bit packing
- [ ] Create bump allocator
- [ ] Implement mark phase (tri-color marking)
- [ ] Implement sweep phase
- [ ] Implement compaction with pointer threading
- [ ] Create handle system (JSGCRef equivalent)
- [ ] Add ROM pointer detection
- [ ] Implement weak reference support
- [ ] Add finalizer support
- [ ] Create comprehensive unit tests
- [ ] Add fuzzing for allocator
- [ ] Performance benchmarks

**Deliverables:**
- Working allocator
- Full GC implementation
- 90%+ test coverage
- Benchmark baseline

**Key Design Decisions:**
- Use indices vs raw pointers (see Section 4.2)
- Interior mutability strategy (see Section 4.3)
- Handle lifetime management (see Section 4.4)

### Phase 2: Value System (Weeks 7-9)

**Goal:** Implement JSValue representation and basic operations.

**Tasks:**
- [ ] Define JSValue tagged union/enum
- [ ] Implement integer encoding/decoding (31-bit with tag)
- [ ] Implement pointer tagging
- [ ] Implement special values (null, undefined, bool, etc.)
- [ ] Add short float support (64-bit platforms)
- [ ] Implement heap-allocated float64
- [ ] Create type checking functions
- [ ] Add type conversion functions
- [ ] Implement string allocation and UTF-8 handling
- [ ] Add string interning (unique strings)
- [ ] Implement value arrays (JSValueArray)
- [ ] Implement byte arrays (JSByteArray)
- [ ] Unit tests for all value types
- [ ] Property-based testing

**Deliverables:**
- Complete JSValue implementation
- String handling
- Array storage
- Test coverage 90%+

### Phase 3: Object System (Weeks 10-14)

**Goal:** Implement objects, prototypes, and properties.

**Tasks:**
- [ ] Define JSObject structure
- [ ] Implement object allocation
- [ ] Create property hash table system
- [ ] Implement property lookup/insertion/deletion
- [ ] Add property rehashing
- [ ] Implement prototype chain traversal
- [ ] Add fast paths for integer array indices
- [ ] Create array objects with special length handling
- [ ] Implement function objects (bytecode and C functions)
- [ ] Add closure support (JSVarRef)
- [ ] Implement error objects with stack traces
- [ ] Add basic typed array support
- [ ] Create class system infrastructure
- [ ] Unit tests for all object operations
- [ ] Integration tests for prototype chains

**Deliverables:**
- Full object system
- Property management
- Prototype inheritance
- 85%+ test coverage

### Phase 4: Bytecode System (Weeks 15-17)

**Goal:** Define bytecode format and instruction encoding.

**Tasks:**
- [ ] Define Opcode enum (~260 opcodes)
- [ ] Implement instruction encoding/decoding
- [ ] Create constant pool management
- [ ] Implement JSFunctionBytecode structure
- [ ] Add bytecode validation
- [ ] Create bytecode disassembler (for debugging)
- [ ] Add debug info (pc2line) with compression
- [ ] Unit tests for each opcode encoding
- [ ] Fuzz testing for bytecode decoder

**Deliverables:**
- Complete bytecode format
- Encoder/decoder
- Debug tools
- Test coverage 90%+

### Phase 5: Virtual Machine (Weeks 18-22)

**Goal:** Implement bytecode interpreter.

**Tasks:**
- [ ] Create VM stack management
- [ ] Implement call frame structure
- [ ] Build opcode dispatch loop (match-based initially)
- [ ] Implement stack operations (dup, drop, swap, etc.)
- [ ] Add value pushing opcodes
- [ ] Implement variable access (local, arg, closure)
- [ ] Add property access opcodes
- [ ] Implement arithmetic operators with fast paths
- [ ] Add comparison operators
- [ ] Implement logical operators
- [ ] Add control flow (jumps, branches)
- [ ] Implement function calls (JS and C functions)
- [ ] Add exception handling (throw/catch)
- [ ] Implement finally blocks (gosub/ret)
- [ ] Add iterator support (for-in/for-of)
- [ ] Implement tail call optimization
- [ ] Add interrupt polling
- [ ] Create VM debugger hooks
- [ ] Unit tests for each opcode
- [ ] Integration tests for opcode combinations
- [ ] Performance benchmarks

**Deliverables:**
- Working bytecode interpreter
- All opcodes implemented
- Exception handling
- Test coverage 85%+
- Performance within 2x of C version

### Phase 6: Compiler (Weeks 23-28)

**Goal:** Implement lexer, parser, and code generator.

**Tasks:**
- [ ] Implement lexer/tokenizer
- [ ] Add Unicode/UTF-8 support in lexer
- [ ] Parse string literals with escapes
- [ ] Parse numeric literals (all formats)
- [ ] Parse regexp literals
- [ ] Implement automatic semicolon insertion
- [ ] Create non-recursive parser state machine
- [ ] Implement expression parsing (precedence climbing)
- [ ] Add statement parsing (if, while, for, etc.)
- [ ] Implement function declaration parsing
- [ ] Add variable declaration parsing
- [ ] Implement class parsing (limited)
- [ ] Create code generation for expressions
- [ ] Add code generation for statements
- [ ] Implement closure variable capture analysis
- [ ] Add constant pool management
- [ ] Implement label management and backpatching
- [ ] Add peephole optimization
- [ ] Implement dead code elimination
- [ ] Create debug info generation
- [ ] Add comprehensive parser tests
- [ ] ECMAScript compliance testing
- [ ] Fuzz testing for parser

**Deliverables:**
- Full parser/compiler
- One-pass compilation
- Optimizations
- Test coverage 85%+
- Pass subset of ECMAScript tests

### Phase 7: Built-in Functions (Weeks 29-34)

**Goal:** Implement JavaScript standard library.

**Tasks:**
- [ ] Global functions (eval, parseInt, parseFloat, etc.)
- [ ] Object methods (create, keys, getPrototypeOf, etc.)
- [ ] Function methods (call, apply, bind)
- [ ] Array methods (push, pop, slice, splice, etc.)
- [ ] Array iteration (forEach, map, filter, reduce)
- [ ] String methods (charAt, slice, indexOf, etc.)
- [ ] String case conversion (ASCII only)
- [ ] String trim methods
- [ ] Number methods (toString, toFixed, etc.)
- [ ] Math functions (all standard functions)
- [ ] Math constants (PI, E, etc.)
- [ ] RegExp basic support
- [ ] JSON.parse and JSON.stringify
- [ ] Error constructors and methods
- [ ] Typed array constructors
- [ ] ArrayBuffer support
- [ ] Unit tests for each built-in
- [ ] Compatibility tests against spec

**Deliverables:**
- Complete built-in library
- ES5 compatibility
- Test coverage 90%+

### Phase 8: Standard Library Build Tool (Weeks 35-36)

**Goal:** Create tool to compile stdlib to ROM structures.

**Tasks:**
- [ ] Parse JavaScript stdlib source
- [ ] Generate bytecode
- [ ] Emit Rust const data structures
- [ ] Implement atom table generation
- [ ] Add C function table generation
- [ ] Create relocation support
- [ ] Integration with build process
- [ ] Optimize for size

**Deliverables:**
- Working build tool
- ROM-resident stdlib
- Instant instantiation

### Phase 9: Optimization and Polish (Weeks 37-40)

**Goal:** Performance tuning and production readiness.

**Tasks:**
- [ ] Profile hot paths
- [ ] Optimize VM dispatch (consider function pointers)
- [ ] Add SIMD optimizations where applicable
- [ ] Optimize GC performance
- [ ] Reduce memory overhead
- [ ] Add comprehensive benchmarks
- [ ] Compare performance to C version
- [ ] Memory profiling
- [ ] Stress testing
- [ ] Documentation polish
- [ ] API ergonomics review
- [ ] Safety audit
- [ ] Consider optional features (no_std, soft-float)

**Deliverables:**
- Performance within 1.2x of C version
- Complete documentation
- Production-ready API
- Benchmark suite

### Phase 10: CLI and Tools (Weeks 41-42)

**Goal:** Create command-line interface and tooling.

**Tasks:**
- [ ] REPL implementation
- [ ] Script execution
- [ ] Bytecode compilation/loading
- [ ] Memory limit control
- [ ] Statistics dumping
- [ ] Readline integration
- [ ] Tab completion
- [ ] Command-line argument parsing
- [ ] User documentation

**Deliverables:**
- Full-featured CLI
- User manual
- Example programs

## 3. Core Abstractions: C to Rust Patterns

### 3.1 Tagged Values → Enums/Unions

**C Pattern:**
```c
typedef uint32_t JSValue;

#define JS_TAG_INT 0
#define JS_TAG_PTR 1
#define JS_VALUE_GET_INT(v) ((int)(v) >> 1)
#define JS_VALUE_TO_PTR(v) ((void *)((v) - 1))
```

**Rust Approach (Option A - Newtype with Manual Tagging):**
```rust
#[repr(transparent)]
pub struct JSValue(usize);

impl JSValue {
    const TAG_MASK: usize = 0x7;
    const TAG_INT: usize = 0;
    const TAG_PTR: usize = 1;
    const TAG_SPECIAL: usize = 3;

    #[inline]
    pub fn from_int(i: i32) -> Self {
        JSValue((i as usize) << 1)
    }

    #[inline]
    pub fn to_int(self) -> Option<i32> {
        if (self.0 & 1) == 0 {
            Some((self.0 as isize >> 1) as i32)
        } else {
            None
        }
    }

    #[inline]
    pub fn from_ptr(idx: HeapIndex) -> Self {
        JSValue((idx.0 << 3) | Self::TAG_PTR)
    }

    #[inline]
    pub fn to_ptr(self) -> Option<HeapIndex> {
        if (self.0 & Self::TAG_MASK) == Self::TAG_PTR {
            Some(HeapIndex(self.0 >> 3))
        } else {
            None
        }
    }
}
```

**Rust Approach (Option B - Safe Enum with Manual Layout):**
```rust
// For pattern matching convenience, but larger size
pub enum JSValue {
    Int(i32),
    Ptr(HeapIndex),
    Null,
    Undefined,
    Bool(bool),
    // ... other variants
}
```

**Recommendation:** Use Option A (newtype) for performance and C-compatible layout. Provide safe wrapper methods for construction and destructuring. Consider Option B for a safe high-level API layer.

### 3.2 Manual Memory Management → Rust Ownership/Arenas

**C Pattern:**
```c
void *js_malloc(JSContext *ctx, uint32_t size, int mtag) {
    JSMemBlockHeader *p = (JSMemBlockHeader *)ctx->heap_free;
    ctx->heap_free += size;
    p->mtag = mtag;
    return p;
}
```

**Rust Approach (Arena with Indices):**
```rust
// Use indices instead of pointers for stability
#[derive(Copy, Clone, Debug)]
pub struct HeapIndex(u32);

pub struct Arena {
    memory: Vec<u8>,
    heap_free: usize,
    stack_bottom: usize,
}

impl Arena {
    pub fn alloc(&mut self, size: usize, mtag: MemTag) -> Result<HeapIndex, OutOfMemory> {
        let size = (size + 7) & !7; // Align to 8 bytes

        if self.heap_free + size > self.stack_bottom {
            return Err(OutOfMemory);
        }

        let index = HeapIndex(self.heap_free as u32);

        // Write header
        unsafe {
            let ptr = self.memory.as_mut_ptr().add(self.heap_free);
            let header = ptr as *mut MemBlockHeader;
            (*header).set_mtag(mtag);
            (*header).set_gc_mark(false);
        }

        self.heap_free += size;
        Ok(index)
    }

    pub fn get<T>(&self, index: HeapIndex) -> &T {
        unsafe {
            let ptr = self.memory.as_ptr().add(index.0 as usize);
            &*(ptr as *const T)
        }
    }

    pub fn get_mut<T>(&mut self, index: HeapIndex) -> &mut T {
        unsafe {
            let ptr = self.memory.as_mut_ptr().add(index.0 as usize);
            &mut *(ptr as *mut T)
        }
    }
}
```

**Key Insight:** Using indices instead of raw pointers eliminates pointer invalidation issues. Indices remain valid across GC compaction. We update the index-to-offset mapping during compaction.

### 3.3 Compacting GC → Safe Rust Alternatives

**Challenge:** Pointer threading requires raw pointer manipulation and violates Rust's aliasing rules.

**Approach 1 - Index-Based (Safest):**
```rust
// Store forwarding addresses in a side table
struct Compactor {
    forwarding_table: HashMap<HeapIndex, usize>,
}

impl Compactor {
    fn compact(&mut self, arena: &mut Arena) {
        // Phase 1: Calculate new addresses
        let mut dst = 0;
        for obj in arena.live_objects() {
            self.forwarding_table.insert(obj.index, dst);
            dst += obj.size;
        }

        // Phase 2: Update all references
        for obj in arena.live_objects_mut() {
            obj.update_references(&self.forwarding_table);
        }

        // Phase 3: Move objects
        for obj in arena.live_objects() {
            let new_addr = self.forwarding_table[&obj.index];
            arena.move_object(obj.index, new_addr);
        }
    }
}
```

**Approach 2 - Unsafe Pointer Threading (Closer to C):**
```rust
unsafe fn thread_pointers(&mut self, arena: &mut Arena) {
    // Mark phase sets "first_field_threaded" bit in header
    // Direct pointer manipulation in unsafe block
    // Document safety invariants extensively

    for obj in arena.marked_objects() {
        let mut thread_head = obj.as_ptr();

        // Walk all references to this object
        for ref_ptr in arena.find_all_refs(obj) {
            let old_val = *ref_ptr;
            *ref_ptr = thread_head as usize;
            thread_head = ref_ptr as *mut usize;
        }

        // Store final value in object
        *thread_head = obj.first_field;
    }
}
```

**Recommendation:** Start with Approach 1 (index-based) for correctness. Profile and optimize. Only switch to Approach 2 if performance requires it, and isolate in well-documented unsafe module.

### 3.4 Pointer Arithmetic → Slices/Indices

**C Pattern:**
```c
JSProperty *props = (JSProperty *)((uint8_t *)array + offset);
props[i].value = val;
```

**Rust Approach:**
```rust
// Flexible array members become separate allocations
pub struct PropertyTable {
    header_index: HeapIndex,
    entries_index: HeapIndex,
}

impl PropertyTable {
    pub fn get(&self, ctx: &Context, i: usize) -> &Property {
        let entries: &[Property] = ctx.arena.get_slice(self.entries_index);
        &entries[i]
    }
}
```

**Alternative - DST (Dynamically Sized Types):**
```rust
#[repr(C)]
pub struct PropertyTable {
    count: u32,
    hash_mask: u32,
    // Cannot directly have flexible array in safe Rust
}

// Instead, allocate extra space and use pointer arithmetic in unsafe
impl PropertyTable {
    unsafe fn entries(&self) -> *const Property {
        let ptr = self as *const Self;
        ptr.add(1) as *const Property
    }
}
```

**Recommendation:** Use separate allocations (first approach) for safety. Document layout carefully if using unsafe approach.

### 3.5 Unions/Bitfields → repr(C) Structs or Enums

**C Pattern:**
```c
struct JSObject {
    uint32_t gc_mark: 1;
    uint32_t class_id: 8;
    union {
        JSClosureData closure;
        JSArrayData array;
    } u;
};
```

**Rust Approach (Option A - Manual Bit Packing):**
```rust
#[repr(C)]
pub struct JSObject {
    header: u32, // Manually pack bits
    proto: JSValue,
    props: JSValue,
    class_data: HeapIndex, // Points to class-specific data
}

impl JSObject {
    const GC_MARK_BIT: u32 = 1 << 0;
    const CLASS_ID_SHIFT: u32 = 1;
    const CLASS_ID_MASK: u32 = 0xFF;

    #[inline]
    pub fn gc_mark(&self) -> bool {
        (self.header & Self::GC_MARK_BIT) != 0
    }

    #[inline]
    pub fn set_gc_mark(&mut self, val: bool) {
        if val {
            self.header |= Self::GC_MARK_BIT;
        } else {
            self.header &= !Self::GC_MARK_BIT;
        }
    }

    #[inline]
    pub fn class_id(&self) -> u8 {
        ((self.header >> Self::CLASS_ID_SHIFT) & Self::CLASS_ID_MASK) as u8
    }
}
```

**Rust Approach (Option B - Tagged Union with Enum):**
```rust
#[repr(C)]
pub struct JSObject {
    header: u32,
    proto: JSValue,
    props: JSValue,
    class_data: ObjectData,
}

#[repr(C)]
pub enum ObjectData {
    Closure(ClosureData),
    Array(ArrayData),
    CFunction(CFunctionData),
    // ...
}
```

**Recommendation:** Use Option A (manual bit packing) for precise control over memory layout. Option B increases size (discriminant overhead) but provides type safety.

### 3.6 C Variadic/Macros → Rust Generics/Macros

**C Pattern:**
```c
#define JS_NewInt32(ctx, val) (((val) >= -0x40000000 && (val) < 0x40000000) ? \
    (JSValue)((val) << 1) : js_new_int32(ctx, val))
```

**Rust Approach:**
```rust
impl Context {
    #[inline]
    pub fn new_int32(&mut self, val: i32) -> JSValue {
        const MIN: i32 = -0x4000_0000;
        const MAX: i32 = 0x3FFF_FFFF;

        if val >= MIN && val <= MAX {
            JSValue::from_int(val)
        } else {
            self.new_int32_slow(val)
        }
    }

    #[cold]
    fn new_int32_slow(&mut self, val: i32) -> JSValue {
        // Allocate boxed integer
        // ...
    }
}
```

**Recommendation:** Use inline functions with `#[cold]` for slow paths. Rust's optimizer handles this well.

## 4. Safety Strategy

### 4.1 Unsafe Code Boundaries

**Principle:** Minimize and isolate unsafe code.

**Safe Modules:**
- Public API (100% safe)
- Compiler/parser (99% safe)
- Built-in functions (100% safe)
- VM interpreter loop (95% safe)

**Unsafe Modules:**
- Memory allocator (mostly unsafe)
- GC implementation (mostly unsafe)
- Bytecode decoding (carefully unsafe)
- Low-level value manipulation (some unsafe)

**Safety Invariants Documentation:**
Every unsafe block must have a comment explaining:
1. Why unsafe is needed
2. What invariants are maintained
3. What could go wrong if misused

Example:
```rust
unsafe {
    // SAFETY: index is valid because:
    // 1. It came from alloc() which returned Ok
    // 2. No GC has occurred since allocation
    // 3. Index points to MemBlockHeader with correct mtag
    let ptr = self.memory.as_ptr().add(index.0 as usize);
    &*(ptr as *const JSObject)
}
```

### 4.2 GC Safety

**Problem:** GC can move objects, invalidating references.

**Solution 1 - Generational Indices:**
```rust
pub struct Handle<T> {
    index: HeapIndex,
    generation: u32,
    _marker: PhantomData<T>,
}

pub struct Arena {
    generation: u32,
    index_map: Vec<usize>, // Maps index to current offset
}

impl Arena {
    pub fn gc(&mut self) {
        // Compact memory
        self.compact();
        // Increment generation to invalidate old handles
        self.generation += 1;
    }

    pub fn get<T>(&self, handle: Handle<T>) -> Option<&T> {
        if handle.generation != self.generation {
            return None; // Stale handle
        }
        // Lookup current offset
        let offset = self.index_map[handle.index.0 as usize];
        // Safe: generation check ensures validity
        unsafe { Some(&*(self.memory.as_ptr().add(offset) as *const T)) }
    }
}
```

**Solution 2 - RAII GC Roots:**
```rust
pub struct GcRoot<'ctx, T> {
    ctx: &'ctx Context,
    slot: *mut JSValue,
    _marker: PhantomData<T>,
}

impl<'ctx, T> GcRoot<'ctx, T> {
    pub fn value(&self) -> JSValue {
        unsafe { *self.slot }
    }
}

impl<'ctx, T> Drop for GcRoot<'ctx, T> {
    fn drop(&mut self) {
        // Remove from GC root stack
        self.ctx.pop_gc_root(self.slot);
    }
}

impl Context {
    pub fn root<T>(&mut self, val: JSValue) -> GcRoot<'_, T> {
        let slot = self.push_gc_root(val);
        GcRoot {
            ctx: self,
            slot,
            _marker: PhantomData,
        }
    }
}

// Usage:
let obj = ctx.root::<JSObject>(ctx.new_object());
ctx.new_string("might GC"); // Safe: obj is rooted
let val = obj.value(); // Still valid
```

**Recommendation:** Use Solution 2 (RAII roots) for ergonomic API. The lifetime `'ctx` ensures roots don't outlive the context.

### 4.3 Interior Mutability

**Problem:** GC needs to mutate objects through shared references.

**Solution - Unsafe Interior Mutability:**
```rust
pub struct Context {
    arena: UnsafeCell<Arena>,
    // Other fields...
}

impl Context {
    fn arena(&self) -> &Arena {
        unsafe { &*self.arena.get() }
    }

    fn arena_mut(&mut self) -> &mut Arena {
        unsafe { &mut *self.arena.get() }
    }

    // Public API takes &mut self
    pub fn gc(&mut self) {
        self.arena_mut().gc();
    }

    // Internal use can mutate through &self (carefully!)
    fn alloc_internal(&self, size: usize) -> HeapIndex {
        unsafe {
            (*self.arena.get()).alloc(size)
        }
    }
}
```

**Safety Reasoning:**
- Public API requires `&mut Context`, preventing aliasing
- Internal code uses `UnsafeCell` for controlled mutation
- Single-threaded only (document this!)
- No concurrent access

### 4.4 Type Safety for Heap Objects

**Problem:** Heap contains heterogeneous objects.

**Solution - Type Tags with Safe Accessors:**
```rust
pub enum MemTag {
    Object,
    String,
    Float64,
    FunctionBytecode,
    ValueArray,
    // ...
}

impl Arena {
    pub fn get_object(&self, idx: HeapIndex) -> Result<&JSObject, TypeError> {
        let header = self.get_header(idx);
        if header.mtag() != MemTag::Object {
            return Err(TypeError);
        }
        unsafe {
            Ok(self.get_unchecked(idx))
        }
    }

    pub fn get_string(&self, idx: HeapIndex) -> Result<&JSString, TypeError> {
        let header = self.get_header(idx);
        if header.mtag() != MemTag::String {
            return Err(TypeError);
        }
        unsafe {
            Ok(self.get_unchecked(idx))
        }
    }
}
```

### 4.5 Bounds Checking

**Principle:** Always check bounds for array access.

```rust
impl Stack {
    pub fn get(&self, offset: isize) -> Result<JSValue, StackUnderflow> {
        let idx = (self.sp as isize + offset) as usize;
        if idx >= self.values.len() {
            return Err(StackUnderflow);
        }
        Ok(self.values[idx])
    }
}
```

**Optimization:** Use `#[cfg(debug_assertions)]` for expensive checks:
```rust
#[inline]
fn get_unchecked(&self, idx: usize) -> JSValue {
    debug_assert!(idx < self.len());
    unsafe { *self.values.get_unchecked(idx) }
}
```

## 5. Testing Strategy

### 5.1 Unit Tests

**Coverage Goal:** 90%+ for all modules

**Test Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_int_encoding() {
        let val = JSValue::from_int(42);
        assert_eq!(val.to_int(), Some(42));
    }

    #[test]
    fn test_gc_simple() {
        let mut ctx = Context::new(1024);
        let obj = ctx.new_object();
        ctx.gc();
        // Object should still be accessible
        assert!(ctx.is_object(obj));
    }
}
```

**Unit Test Categories:**
- Value encoding/decoding
- Memory allocation/deallocation
- GC marking and compaction
- Property insertion/lookup/deletion
- Type conversions
- Each opcode independently
- String operations
- Array operations

### 5.2 Integration Tests

**Test Suite Location:** `tests/integration/`

**Categories:**
- Parse and execute simple scripts
- Function calls and returns
- Closure capture
- Exception handling
- Prototype chains
- Built-in function behavior
- Memory limits
- GC stress tests

**Example:**
```rust
#[test]
fn test_closure_capture() {
    let mut ctx = Context::new(8192);
    let script = r#"
        function outer(x) {
            return function inner(y) {
                return x + y;
            };
        }
        var add5 = outer(5);
        add5(3);
    "#;
    let result = ctx.eval(script).unwrap();
    assert_eq!(result.to_int(), Some(8));
}
```

### 5.3 Compliance Tests

**Goal:** Pass subset of ECMAScript 5.1 test suite

**Test Organization:**
```
tests/compliance/
├── expressions/
├── statements/
├── functions/
├── objects/
├── arrays/
└── builtins/
```

**Approach:**
1. Start with Test262 ES5 subset
2. Filter for supported features
3. Automate test running
4. Track pass/fail rates
5. Fix failures iteratively

**Target:** 80%+ pass rate for ES5 strict mode tests

### 5.4 Property-Based Testing

**Use `proptest` for invariant testing:**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_int_roundtrip(i in -0x4000_0000i32..0x4000_0000i32) {
        let val = JSValue::from_int(i);
        prop_assert_eq!(val.to_int(), Some(i));
    }

    #[test]
    fn test_gc_preserves_reachable(ops in vec(gc_operation(), 1..100)) {
        let mut ctx = Context::new(8192);
        let root = ctx.new_object();
        ctx.root(root);

        for op in ops {
            apply_operation(&mut ctx, op);
        }

        ctx.gc();
        prop_assert!(ctx.is_valid(root));
    }
}
```

### 5.5 Fuzzing

**Fuzz Targets:**
- Parser (feed random byte sequences)
- Bytecode decoder
- VM interpreter
- GC allocator

**Setup with cargo-fuzz:**
```rust
// fuzz/fuzz_targets/parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut ctx = Context::new(8192);
        let _ = ctx.parse(s, "fuzz.js", 0);
    }
});
```

**Continuous Fuzzing:**
- Run on CI for 10+ minutes per target
- Track corpus in git
- File issues for crashes

### 5.6 Performance Testing

**Benchmark Suite with Criterion:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_fibonacci(c: &mut Criterion) {
    let mut ctx = Context::new(8192);
    let script = r#"
        function fib(n) {
            if (n < 2) return n;
            return fib(n-1) + fib(n-2);
        }
        fib(20);
    "#;

    c.bench_function("fibonacci", |b| {
        b.iter(|| {
            ctx.eval(black_box(script)).unwrap()
        })
    });
}

criterion_group!(benches, bench_fibonacci);
criterion_main!(benches);
```

**Benchmark Categories:**
- Microbenchmarks (individual operations)
- Macro benchmarks (real programs)
- Memory allocation speed
- GC pause time
- Parser throughput
- VM dispatch overhead

**Performance Goals:**
- Within 1.2x of C version for most benchmarks
- GC pause < 2ms for 64KB heap
- Parser throughput > 1 MB/s

### 5.7 Memory Testing

**Valgrind/MIRI:**
```bash
cargo miri test
```

**Custom Memory Debugger:**
```rust
#[cfg(debug_assertions)]
impl Arena {
    fn check_invariants(&self) {
        // Verify heap structure
        // Check for overlapping blocks
        // Validate free list
        // etc.
    }
}
```

**Leak Detection:**
```rust
#[test]
fn test_no_leaks() {
    let initial = Context::memory_usage();
    {
        let mut ctx = Context::new(8192);
        // Use context...
    }
    let final_usage = Context::memory_usage();
    assert_eq!(initial, final_usage);
}
```

## 6. Dependencies

### 6.1 Core Library: Zero Dependencies

**Rationale:**
- Maintain minimal footprint
- Reduce attack surface
- Ensure predictable behavior
- Enable no_std support
- Simplify auditing

**All Functionality Implemented Natively:**
- UTF-8 handling
- Hash tables for properties
- Number parsing/formatting (dtoa)
- Math functions (basic libm)
- Regular expressions (basic engine)

### 6.2 Development Dependencies

**Testing:**
```toml
[dev-dependencies]
proptest = "1.4"        # Property-based testing
criterion = "0.5"       # Benchmarking
quickcheck = "1.0"      # Alternative property testing
```

**Tooling:**
```toml
[build-dependencies]
# None - keep build simple
```

### 6.3 Optional Feature Dependencies

**CLI Features:**
```toml
[dependencies]
rustyline = { version = "13.0", optional = true }

[features]
default = []
repl = ["rustyline"]
```

### 6.4 Future Considerations

**Potentially Useful Crates (Evaluate Later):**

**hashbrown** - High-performance hash table
- Pro: Faster than std HashMap
- Con: Adds dependency, we need custom layout anyway
- Decision: Implement custom hash table first, benchmark later

**bumpalo** - Bump allocator
- Pro: Well-tested arena allocator
- Con: Doesn't support compaction, we need custom GC
- Decision: Reference implementation only, use custom

**bytemuck** - Safe transmutation
- Pro: Safe casting for bytecode
- Con: Can implement ourselves easily
- Decision: Maybe for convenience, not critical

**bitflags** - Bit flag macros
- Pro: Nice API for flags
- Con: Simple to implement
- Decision: Implement manually

**Recommendation:** Start with zero dependencies. Revisit after profiling if specific crates show clear wins.

## 7. Risk Assessment

### 7.1 Technical Risks

#### Risk 1: GC Complexity (HIGH)

**Challenge:** Pointer threading compaction is complex and error-prone.

**Impact:** Correctness bugs, memory corruption, hard-to-debug crashes.

**Mitigation:**
- Start with simpler GC (mark-sweep without compaction)
- Extensive testing before adding compaction
- Fuzz testing
- Memory debugging tools (MIRI)
- Consider index-based approach first
- Reference C implementation closely
- Add validation checks in debug builds

**Contingency:** If compaction proves too difficult, use simpler copying collector or no compaction (accept fragmentation).

#### Risk 2: Performance Regression (MEDIUM)

**Challenge:** Rust safety checks might slow execution.

**Impact:** 2-3x slower than C version, defeating purpose.

**Mitigation:**
- Profile early and often
- Use `#[inline]` aggressively
- Optimize hot paths (VM dispatch)
- Consider unsafe in critical sections
- Benchmark against C version regularly
- Target release builds, not debug

**Contingency:** Document performance tradeoffs. Provide unsafe fast paths as opt-in features.

#### Risk 3: Memory Overhead (MEDIUM)

**Challenge:** Rust enum discriminants, padding, alignment.

**Impact:** Larger memory footprint, can't hit 10 kB target.

**Mitigation:**
- Use manual tagging instead of enums where needed
- `#[repr(C)]` and `#[repr(packed)]` carefully
- Profile memory layout
- Compare to C struct sizes
- Use bitpacking extensively

**Contingency:** Document minimum RAM requirement (may be higher than C version).

#### Risk 4: Unsafe Code Bugs (HIGH)

**Challenge:** Memory-unsafe bugs in GC, allocator, VM.

**Impact:** Undefined behavior, crashes, security vulnerabilities.

**Mitigation:**
- Minimize unsafe code surface
- Extensive documentation of invariants
- MIRI testing
- Fuzz testing
- Code review focused on unsafe blocks
- Consider formal verification for critical sections

**Contingency:** Provide safe-only mode (slower but guaranteed safe).

#### Risk 5: Parser Complexity (MEDIUM)

**Challenge:** Non-recursive parser is complex state machine.

**Impact:** Hard to implement correctly, maintain, debug.

**Mitigation:**
- Allow recursion initially (simpler)
- Stack size limits
- Convert to iterative later if needed
- Reference C implementation
- Extensive parser tests

**Contingency:** Accept limited recursion depth with clear error messages.

### 7.2 Project Risks

#### Risk 1: Scope Creep (MEDIUM)

**Challenge:** Temptation to add features beyond MicroQuickJS.

**Impact:** Never finish, lose focus on minimal footprint.

**Mitigation:**
- Strict adherence to MicroQuickJS feature set
- Defer enhancements to post-1.0
- Track scope changes in ADRs
- Regular scope reviews

#### Risk 2: Time Estimation (MEDIUM)

**Challenge:** 6-9 months might be optimistic.

**Impact:** Project drags on, loses momentum.

**Mitigation:**
- Start with MVP (phases 1-5)
- Release early versions
- Accept incomplete stdlib initially
- Prioritize ruthlessly

**Contingency:** Ship reduced feature set as 0.5, iterate.

#### Risk 3: Compatibility (LOW)

**Challenge:** Behavior differs from C version.

**Impact:** Users expect exact compatibility.

**Mitigation:**
- Extensive compliance testing
- Document differences clearly
- Provide compatibility mode
- Cross-reference test suites

### 7.3 Dependency Risks

**Risk:** Zero-dependency strategy increases implementation burden.

**Impact:** Slower development, potential bugs in custom implementations.

**Mitigation:**
- Reference high-quality implementations
- Test thoroughly
- Document algorithms
- Consider selective dependencies if justified

## 8. Detailed Task Breakdown

### 8.1 Phase 1: Memory Management (Weeks 3-6)

#### Week 3: Memory Block Headers and Allocator

**Tasks:**
- [ ] Define `MemTag` enum
- [ ] Implement `MemBlockHeader` with bitpacking
  - [ ] `gc_mark: bool` (1 bit)
  - [ ] `mtag: MemTag` (3 bits)
  - [ ] Getters/setters for bit fields
- [ ] Create `Arena` struct
  - [ ] `memory: Vec<u8>`
  - [ ] `heap_free: usize`
  - [ ] `stack_bottom: usize`
- [ ] Implement `Arena::new(size)`
- [ ] Implement `Arena::alloc(size, mtag)`
  - [ ] Alignment to 8 bytes
  - [ ] Out-of-memory check
  - [ ] Bump pointer update
  - [ ] Header initialization
- [ ] Implement `Arena::free(index)` (for last block only)
- [ ] Implement `Arena::shrink(index, new_size)`
- [ ] Write unit tests
  - [ ] Test alignment
  - [ ] Test allocation
  - [ ] Test free
  - [ ] Test out-of-memory

**Deliverable:** Working bump allocator with tests.

#### Week 4: GC Marking Phase

**Tasks:**
- [ ] Define `GCMarkState` struct
- [ ] Implement mark stack in free heap space
- [ ] Create `Context::gc_mark_roots()`
  - [ ] Mark current exception
  - [ ] Mark global object
  - [ ] Mark VM stack values
  - [ ] Mark GC root handles
- [ ] Implement `Context::gc_mark_value(val)`
  - [ ] Handle integers (skip)
  - [ ] Handle pointers (mark object)
  - [ ] Handle special values (skip)
- [ ] Implement `Context::gc_mark_object(idx)`
  - [ ] Check if already marked
  - [ ] Set gc_mark bit
  - [ ] Push references to mark stack
  - [ ] Handle mark stack overflow
- [ ] Implement mark stack overflow recovery
- [ ] Write unit tests
  - [ ] Test simple marking
  - [ ] Test reachability
  - [ ] Test mark stack overflow
  - [ ] Test weak references

**Deliverable:** Working mark phase with tests.

#### Week 5: GC Sweep and Compaction

**Tasks:**
- [ ] Implement `Arena::sweep()`
  - [ ] Walk heap
  - [ ] Clear gc_mark on live objects
  - [ ] Call finalizers on dead objects
  - [ ] Mark dead blocks as free
- [ ] Implement weak reference clearing
  - [ ] Unique strings table
  - [ ] String position cache
- [ ] Design compaction strategy (index-based vs pointer threading)
- [ ] Implement `Arena::compact()` - index-based approach
  - [ ] Build forwarding table
  - [ ] Update all references
  - [ ] Move objects
  - [ ] Update heap_free
- [ ] Write unit tests
  - [ ] Test sweep
  - [ ] Test compaction
  - [ ] Test reference updates
  - [ ] Test no leaks

**Deliverable:** Complete GC with compaction.

#### Week 6: GC Handles and Testing

**Tasks:**
- [ ] Define `GcRoot<'ctx, T>` struct
- [ ] Implement `Context::root(val) -> GcRoot`
- [ ] Implement `GcRoot::value()`
- [ ] Implement `Drop for GcRoot` (unroot)
- [ ] Create GC root stack in Context
- [ ] Integrate roots with marking
- [ ] Add comprehensive GC tests
  - [ ] Test RAII roots
  - [ ] Test nested allocations
  - [ ] Test root lifetime
  - [ ] Stress test with random allocations
- [ ] Add fuzzing for allocator
- [ ] Create benchmarks
  - [ ] Allocation speed
  - [ ] GC pause time
  - [ ] Memory overhead
- [ ] Profile and optimize

**Deliverable:** Production-ready memory management with 90%+ test coverage.

### 8.2 Phase 2: Value System (Weeks 7-9)

#### Week 7: JSValue Core

**Tasks:**
- [ ] Define `JSValue` newtype
- [ ] Implement integer encoding
  - [ ] `from_int(i32) -> JSValue`
  - [ ] `to_int() -> Option<i32>`
  - [ ] Test 31-bit range
- [ ] Implement pointer encoding
  - [ ] `from_ptr(HeapIndex) -> JSValue`
  - [ ] `to_ptr() -> Option<HeapIndex>`
  - [ ] Tag bits: 0b001
- [ ] Implement special values
  - [ ] `null() -> JSValue`
  - [ ] `undefined() -> JSValue`
  - [ ] `bool(bool) -> JSValue`
  - [ ] `exception() -> JSValue`
- [ ] Implement type checking
  - [ ] `is_int()`
  - [ ] `is_ptr()`
  - [ ] `is_null()`
  - [ ] `is_undefined()`
  - [ ] `is_bool()`
- [ ] Add 64-bit short float support (conditional)
- [ ] Write unit tests
  - [ ] Test all encodings
  - [ ] Test type checking
  - [ ] Property-based tests

**Deliverable:** Complete JSValue implementation.

#### Week 8: Strings

**Tasks:**
- [ ] Define `JSString` struct
  - [ ] Header with bitpacking
  - [ ] `is_unique: bool`
  - [ ] `is_ascii: bool`
  - [ ] `is_numeric: bool`
  - [ ] `len: u29` (UTF-8 byte length)
  - [ ] Variable UTF-8 data
- [ ] Implement `Context::new_string(s: &str)`
  - [ ] Allocate JSString
  - [ ] Copy UTF-8 data
  - [ ] Set flags
- [ ] Implement `Context::get_string(val) -> &str`
- [ ] Implement string interning
  - [ ] Unique strings table (sorted array)
  - [ ] Binary search
  - [ ] Insertion
  - [ ] Weak references
- [ ] Implement UTF-8 utilities
  - [ ] Byte length
  - [ ] Character count
  - [ ] Index conversions
- [ ] Add single-character optimization (TAG_STRING_CHAR)
- [ ] Write unit tests
  - [ ] Test allocation
  - [ ] Test interning
  - [ ] Test UTF-8 handling
  - [ ] Test ASCII detection

**Deliverable:** Full string support.

#### Week 9: Arrays and Float64

**Tasks:**
- [ ] Define `JSValueArray` struct
  - [ ] Header
  - [ ] Size field
  - [ ] Variable JSValue data
- [ ] Implement `Context::alloc_value_array(size)`
- [ ] Implement `Context::resize_value_array(idx, new_size)`
- [ ] Define `JSByteArray` struct
- [ ] Implement `Context::alloc_byte_array(size)`
- [ ] Implement `Context::resize_byte_array(idx, new_size)`
- [ ] Implement float64 boxing
  - [ ] `Context::new_float64(f64) -> JSValue`
  - [ ] `Context::get_float64(val) -> Option<f64>`
  - [ ] Heap allocation for large exponents
- [ ] Implement numeric conversion
  - [ ] `to_number(val) -> Result<f64>`
  - [ ] `to_int32(val) -> Result<i32>`
  - [ ] `to_uint32(val) -> Result<u32>`
- [ ] Write unit tests
  - [ ] Test arrays
  - [ ] Test resizing
  - [ ] Test float64
  - [ ] Test conversions

**Deliverable:** Complete value system with 90%+ coverage.

### 8.3 Phase 3: Object System (Weeks 10-14)

#### Week 10: Basic Objects

**Tasks:**
- [ ] Define `JSObject` struct
  - [ ] Header with class_id
  - [ ] `proto: JSValue`
  - [ ] `props: JSValue` (JSValueArray index)
  - [ ] `class_data: HeapIndex`
- [ ] Implement `Context::new_object() -> JSValue`
- [ ] Implement `Context::get_object(val) -> Result<&JSObject>`
- [ ] Define class ID constants
  - [ ] `JS_CLASS_OBJECT`
  - [ ] `JS_CLASS_ARRAY`
  - [ ] `JS_CLASS_FUNCTION`
  - [ ] etc.
- [ ] Implement prototype access
  - [ ] `Context::get_prototype(obj) -> JSValue`
  - [ ] `Context::set_prototype(obj, proto)`
- [ ] Write unit tests
  - [ ] Test object creation
  - [ ] Test prototype chains

**Deliverable:** Basic object infrastructure.

#### Week 11: Property System - Hash Tables

**Tasks:**
- [ ] Define `Property` struct
  - [ ] `key: JSValue`
  - [ ] `value: JSValue`
  - [ ] `hash_next: u30`
  - [ ] `prop_type: u2`
- [ ] Define property types
  - [ ] `PROP_NORMAL`
  - [ ] `PROP_GETSET`
  - [ ] `PROP_VARREF`
  - [ ] `PROP_SPECIAL`
- [ ] Implement property storage format
  - [ ] Count field
  - [ ] Hash mask
  - [ ] Hash table (indices)
  - [ ] Property array
- [ ] Implement `Context::alloc_props(initial_size)`
- [ ] Implement hash function for keys
- [ ] Write unit tests
  - [ ] Test hash function
  - [ ] Test allocation

**Deliverable:** Property storage format.

#### Week 12: Property Operations

**Tasks:**
- [ ] Implement `Context::find_own_property(obj, key)`
  - [ ] Hash key
  - [ ] Probe hash table
  - [ ] Walk chain
  - [ ] Return property or None
- [ ] Implement `Context::add_property(obj, key, val)`
  - [ ] Find insertion point
  - [ ] Resize if needed
  - [ ] Insert property
  - [ ] Update hash table
- [ ] Implement `Context::delete_property(obj, key)`
  - [ ] Find property
  - [ ] Remove from hash chain
  - [ ] Mark deleted (or compact)
- [ ] Implement `Context::rehash_props(obj)`
  - [ ] Rebuild hash table
  - [ ] Compact deleted entries
- [ ] Write unit tests
  - [ ] Test insertion
  - [ ] Test lookup
  - [ ] Test deletion
  - [ ] Test collisions
  - [ ] Test resizing

**Deliverable:** Working property hash tables.

#### Week 13: Property Access and Arrays

**Tasks:**
- [ ] Implement `Context::get_property(obj, key) -> Result<JSValue>`
  - [ ] Check own properties
  - [ ] Walk prototype chain
  - [ ] Handle special properties
- [ ] Implement `Context::set_property(obj, key, val) -> Result<()>`
  - [ ] Check for setter
  - [ ] Add if not exists
  - [ ] Update if exists
- [ ] Implement fast paths
  - [ ] Integer array indices
  - [ ] String keys
- [ ] Define `JSArray` class data
  - [ ] `tab: JSValue` (JSValueArray or null)
  - [ ] `len: u32`
- [ ] Implement `Context::new_array(len) -> JSValue`
- [ ] Implement array length handling
  - [ ] Get length
  - [ ] Set length (truncate/extend)
- [ ] Write unit tests
  - [ ] Test property access
  - [ ] Test prototype chain
  - [ ] Test arrays
  - [ ] Test array length

**Deliverable:** Full property and array support.

#### Week 14: Functions and Closures

**Tasks:**
- [ ] Define `JSFunctionBytecode` struct
  - [ ] `func_name: JSValue`
  - [ ] `byte_code: JSValue` (JSByteArray)
  - [ ] `cpool: JSValue` (JSValueArray)
  - [ ] `vars: JSValue`
  - [ ] `ext_vars: JSValue`
  - [ ] `stack_size: u16`
  - [ ] `arg_count: u16`
  - [ ] `filename: JSValue`
  - [ ] `pc2line: JSValue`
- [ ] Define `JSClosureData` struct
  - [ ] `func_bytecode: JSValue`
  - [ ] `var_refs: HeapIndex` (array of JSVarRef)
- [ ] Define `JSVarRef` struct
  - [ ] `is_detached: bool`
  - [ ] Union: value or (next, pvalue)
- [ ] Implement `Context::new_closure(bytecode) -> JSValue`
- [ ] Implement C function support
  - [ ] `JSCFunctionData` struct
  - [ ] `c_function_table`
- [ ] Write unit tests
  - [ ] Test function creation
  - [ ] Test closures
  - [ ] Test C functions

**Deliverable:** Complete object system with 85%+ coverage.

### 8.4 Phase 4: Bytecode System (Weeks 15-17)

#### Week 15: Opcode Definitions

**Tasks:**
- [ ] Define `Opcode` enum with ~260 variants
  - [ ] Stack ops: drop, dup, swap, etc.
  - [ ] Push ops: push_i8, push_const, etc.
  - [ ] Variable ops: get_loc, put_loc, etc.
  - [ ] Property ops: get_field, put_field, etc.
  - [ ] Arithmetic ops: add, sub, mul, etc.
  - [ ] Comparison ops: lt, eq, etc.
  - [ ] Logical ops: not, and, or, etc.
  - [ ] Control flow: if_false, goto, return, etc.
  - [ ] Call ops: call, call_method, etc.
  - [ ] Other: fclosure, typeof, delete, etc.
- [ ] Define instruction formats
  - [ ] `none`, `u8`, `i8`, `u16`, `i16`, `u32`, `label`, `const16`
- [ ] Create opcode metadata table
  - [ ] Format
  - [ ] Stack effect
  - [ ] Name (for debugging)
- [ ] Write unit tests
  - [ ] Test enum sizes
  - [ ] Test metadata

**Deliverable:** Complete opcode definitions.

#### Week 16: Instruction Encoding/Decoding

**Tasks:**
- [ ] Implement bytecode encoder
  - [ ] `Bytecode::emit_op(opcode)`
  - [ ] `Bytecode::emit_u8(val)`
  - [ ] `Bytecode::emit_u16(val)`
  - [ ] `Bytecode::emit_u32(val)`
  - [ ] `Bytecode::emit_label(label_id)`
- [ ] Implement bytecode decoder
  - [ ] `Bytecode::decode(pc) -> (Opcode, operands, new_pc)`
  - [ ] Handle variable-length instructions
  - [ ] Bounds checking
- [ ] Implement label management
  - [ ] `LabelId` type
  - [ ] `new_label() -> LabelId`
  - [ ] `label_here(label)`
  - [ ] Backpatching
- [ ] Implement constant pool
  - [ ] Add constant
  - [ ] Deduplicate constants
  - [ ] Get constant
- [ ] Write unit tests
  - [ ] Test encoding/decoding
  - [ ] Test labels
  - [ ] Test constant pool
  - [ ] Fuzz decoder

**Deliverable:** Bytecode encoder/decoder.

#### Week 17: Debug Info and Validation

**Tasks:**
- [ ] Implement pc2line compression
  - [ ] Exponential-Golomb encoding
  - [ ] Delta encoding
  - [ ] Encode line/column
- [ ] Implement pc2line decompression
  - [ ] Decode to (line, column)
  - [ ] Binary search
- [ ] Implement bytecode validator
  - [ ] Check valid opcodes
  - [ ] Check label targets
  - [ ] Check stack depths
  - [ ] Check constant indices
- [ ] Implement bytecode disassembler
  - [ ] Print opcodes
  - [ ] Show operands
  - [ ] Annotate with line numbers
- [ ] Write unit tests
  - [ ] Test compression
  - [ ] Test validation
  - [ ] Test disassembler

**Deliverable:** Complete bytecode system with debug tools, 90%+ coverage.

### 8.5 Phase 5: Virtual Machine (Weeks 18-22)

#### Week 18: Stack and Frames

**Tasks:**
- [ ] Define `Stack` struct
  - [ ] Values array in arena
  - [ ] Stack pointer
  - [ ] Stack top
  - [ ] Frame pointer
- [ ] Implement stack operations
  - [ ] `push(val)`
  - [ ] `pop() -> JSValue`
  - [ ] `get(offset) -> JSValue`
  - [ ] `set(offset, val)`
  - [ ] Bounds checking
- [ ] Define frame layout constants
  - [ ] `FRAME_OFFSET_PREV_FP`
  - [ ] `FRAME_OFFSET_CALL_FLAGS`
  - [ ] `FRAME_OFFSET_CUR_PC`
  - [ ] `FRAME_OFFSET_THIS`
  - [ ] `FRAME_OFFSET_FUNC_OBJ`
  - [ ] `FRAME_OFFSET_ARG0`
- [ ] Implement frame management
  - [ ] `push_frame(argc)`
  - [ ] `pop_frame()`
  - [ ] `get_local(idx)`
  - [ ] `set_local(idx, val)`
  - [ ] `get_arg(idx)`
- [ ] Write unit tests
  - [ ] Test stack ops
  - [ ] Test frames
  - [ ] Test stack overflow

**Deliverable:** Stack and frame management.

#### Week 19: VM Core and Basic Opcodes

**Tasks:**
- [ ] Define `VM` struct
  - [ ] Context reference
  - [ ] Program counter
  - [ ] Bytecode reference
- [ ] Implement VM dispatch loop
  - [ ] Match-based dispatch
  - [ ] Fetch opcode
  - [ ] Decode operands
  - [ ] Execute
  - [ ] Loop
- [ ] Implement stack manipulation opcodes
  - [ ] `OP_drop`
  - [ ] `OP_dup`
  - [ ] `OP_swap`
  - [ ] `OP_nip`
  - [ ] `OP_insert2`, `OP_insert3`
  - [ ] `OP_perm3`, `OP_rot3l`
- [ ] Implement push opcodes
  - [ ] `OP_push_value`
  - [ ] `OP_push_i8`, `OP_push_i16`
  - [ ] `OP_push_const8`, `OP_push_const16`
  - [ ] `OP_undefined`, `OP_null`
  - [ ] `OP_push_true`, `OP_push_false`
- [ ] Write unit tests for each opcode

**Deliverable:** VM core with basic opcodes.

#### Week 20: Variable and Property Opcodes

**Tasks:**
- [ ] Implement variable access opcodes
  - [ ] `OP_get_loc`, `OP_put_loc`
  - [ ] `OP_get_arg`, `OP_put_arg`
  - [ ] `OP_get_var_ref`, `OP_put_var_ref`
  - [ ] Fast variants (loc0-loc3)
- [ ] Implement property access opcodes
  - [ ] `OP_get_field`, `OP_put_field`
  - [ ] `OP_get_array_el`, `OP_put_array_el`
  - [ ] `OP_get_length`
  - [ ] `OP_define_field`
  - [ ] Fast paths for integers and strings
- [ ] Write unit tests
  - [ ] Test variable access
  - [ ] Test property access
  - [ ] Test fast paths

**Deliverable:** Variable and property opcodes.

#### Week 21: Arithmetic and Control Flow

**Tasks:**
- [ ] Implement arithmetic opcodes
  - [ ] `OP_add`, `OP_sub`, `OP_mul`, `OP_div`, `OP_mod`
  - [ ] `OP_inc`, `OP_dec`, `OP_neg`, `OP_plus`
  - [ ] `OP_shl`, `OP_sar`, `OP_shr`
  - [ ] `OP_pow`
  - [ ] Fast paths for integers
  - [ ] Overflow handling
- [ ] Implement comparison opcodes
  - [ ] `OP_lt`, `OP_lte`, `OP_gt`, `OP_gte`
  - [ ] `OP_eq`, `OP_neq`
  - [ ] `OP_strict_eq`, `OP_strict_neq`
  - [ ] `OP_instanceof`, `OP_in`
- [ ] Implement logical opcodes
  - [ ] `OP_not`, `OP_lnot`
  - [ ] `OP_and`, `OP_or`, `OP_xor`
- [ ] Implement control flow opcodes
  - [ ] `OP_if_false`, `OP_if_true`
  - [ ] `OP_goto`
  - [ ] `OP_return`, `OP_return_undef`
- [ ] Write unit tests for each opcode

**Deliverable:** Arithmetic and control flow.

#### Week 22: Function Calls and Exceptions

**Tasks:**
- [ ] Implement call opcodes
  - [ ] `OP_call`
  - [ ] `OP_call_constructor`
  - [ ] `OP_call_method`
  - [ ] Handle JS functions
  - [ ] Handle C functions
  - [ ] Argument count mismatch
  - [ ] Tail call optimization
- [ ] Implement closure opcode
  - [ ] `OP_fclosure`
  - [ ] Capture variables
- [ ] Implement exception opcodes
  - [ ] `OP_throw`
  - [ ] `OP_catch`
  - [ ] Stack unwinding
  - [ ] Exception propagation
- [ ] Implement finally opcodes
  - [ ] `OP_gosub`
  - [ ] `OP_ret`
- [ ] Implement other opcodes
  - [ ] `OP_typeof`
  - [ ] `OP_delete`
  - [ ] `OP_regexp`
  - [ ] `OP_array_from`
  - [ ] Iterators
- [ ] Add interrupt polling
- [ ] Write comprehensive tests
  - [ ] Test calls
  - [ ] Test exceptions
  - [ ] Test finally
  - [ ] Integration tests

**Deliverable:** Complete VM with all opcodes, 85%+ coverage.

### 8.6 Phase 6: Compiler (Weeks 23-28)

#### Week 23: Lexer

**Tasks:**
- [ ] Define `Token` enum
  - [ ] Keywords
  - [ ] Operators
  - [ ] Literals (number, string, regexp)
  - [ ] Identifiers
  - [ ] Punctuation
- [ ] Define `Lexer` struct
  - [ ] Source buffer
  - [ ] Position
  - [ ] Current line/column
- [ ] Implement `Lexer::next_token() -> Result<Token>`
- [ ] Implement identifier parsing
  - [ ] Keyword detection
  - [ ] UTF-8 support
- [ ] Implement operator parsing
  - [ ] All JS operators
  - [ ] Multi-character operators (==, !=, ===, etc.)
- [ ] Implement numeric literal parsing
  - [ ] Decimal, hex, binary, octal
  - [ ] Floating point
  - [ ] Exponents
- [ ] Write unit tests
  - [ ] Test each token type
  - [ ] Test edge cases
  - [ ] Fuzz testing

**Deliverable:** Complete lexer.

#### Week 24: String and Regexp Parsing

**Tasks:**
- [ ] Implement string literal parsing
  - [ ] Single and double quotes
  - [ ] Escape sequences (`\n`, `\t`, `\\`, etc.)
  - [ ] Unicode escapes (`\uXXXX`, `\u{XXXX}`)
  - [ ] Hex escapes (`\xXX`)
  - [ ] Line continuations
- [ ] Implement template literal parsing (basic)
  - [ ] Backtick strings
  - [ ] Substitutions
- [ ] Implement regexp literal parsing
  - [ ] Pattern
  - [ ] Flags (i, m, g, s, u, y)
- [ ] Implement automatic semicolon insertion
  - [ ] Line terminator tracking
  - [ ] Insertion rules
- [ ] Write unit tests
  - [ ] Test string parsing
  - [ ] Test escapes
  - [ ] Test templates
  - [ ] Test regexps
  - [ ] Test ASI

**Deliverable:** Complete lexer with all literal types.

#### Week 25: Parser Infrastructure

**Tasks:**
- [ ] Define `Parser` struct
  - [ ] Lexer
  - [ ] Current token
  - [ ] Parse state
  - [ ] Current function bytecode
  - [ ] Label stack
  - [ ] Loop stack
- [ ] Implement token management
  - [ ] `peek_token()`
  - [ ] `consume_token()`
  - [ ] `expect(token_type)`
  - [ ] `match_token(token_type)`
- [ ] Implement error reporting
  - [ ] Syntax errors
  - [ ] Line/column info
  - [ ] Error recovery (optional)
- [ ] Create non-recursive state machine
  - [ ] Parse stack
  - [ ] State enum
  - [ ] Loop-based parsing
- [ ] Write unit tests

**Deliverable:** Parser infrastructure.

#### Week 26: Expression Parsing

**Tasks:**
- [ ] Implement precedence climbing
  - [ ] Operator precedence table
  - [ ] `parse_binary_expr(min_prec)`
- [ ] Implement primary expressions
  - [ ] Literals (number, string, bool, null, undefined)
  - [ ] Identifiers
  - [ ] Array literals
  - [ ] Object literals
  - [ ] Function expressions
  - [ ] Parenthesized expressions
- [ ] Implement postfix expressions
  - [ ] Member access (`.`, `[]`)
  - [ ] Function calls
  - [ ] `new`
  - [ ] `++`, `--`
- [ ] Implement unary expressions
  - [ ] `typeof`, `delete`, `void`
  - [ ] `+`, `-`, `!`, `~`
  - [ ] `++`, `--` (prefix)
- [ ] Implement binary expressions
  - [ ] All operators
  - [ ] Precedence
  - [ ] Associativity
- [ ] Implement conditional expression (`? :`)
- [ ] Implement assignment expressions
- [ ] Write unit tests for each expression type

**Deliverable:** Complete expression parser.

#### Week 27: Statement Parsing

**Tasks:**
- [ ] Implement statement parsing
  - [ ] Expression statement
  - [ ] Block statement
  - [ ] Variable declaration (`var`, `let`, `const`)
  - [ ] If/else statement
  - [ ] While loop
  - [ ] Do-while loop
  - [ ] For loop
  - [ ] For-in loop
  - [ ] For-of loop (basic)
  - [ ] Switch statement
  - [ ] Break/continue
  - [ ] Return
  - [ ] Throw
  - [ ] Try/catch/finally
  - [ ] Empty statement
  - [ ] Debugger statement
- [ ] Implement function declaration parsing
  - [ ] Parameters
  - [ ] Body
  - [ ] Closures
- [ ] Implement class declaration (limited)
- [ ] Write unit tests for each statement type

**Deliverable:** Complete statement parser.

#### Week 28: Code Generation and Optimization

**Tasks:**
- [ ] Implement code generation for expressions
  - [ ] Emit opcodes
  - [ ] Manage stack depth
  - [ ] Add constants to pool
- [ ] Implement code generation for statements
  - [ ] Control flow
  - [ ] Loops
  - [ ] Exceptions
- [ ] Implement variable management
  - [ ] Local variable allocation
  - [ ] Closure variable capture
  - [ ] Variable references
- [ ] Implement peephole optimization
  - [ ] Remove redundant ops
  - [ ] Constant folding
  - [ ] Combine ops
- [ ] Implement dead code elimination
  - [ ] Track reachability
  - [ ] Skip unreachable code
- [ ] Implement debug info generation
  - [ ] Line numbers
  - [ ] Column numbers (optional)
  - [ ] Compression
- [ ] Write comprehensive tests
  - [ ] Test compilation
  - [ ] Test optimizations
  - [ ] ECMAScript compliance
  - [ ] Fuzz testing

**Deliverable:** Complete parser/compiler with optimizations, 85%+ coverage.

### 8.7 Phase 7: Built-in Functions (Weeks 29-34)

#### Week 29: Global and Object Built-ins

**Tasks:**
- [ ] Implement global functions
  - [ ] `eval()`
  - [ ] `isNaN()`, `isFinite()`
  - [ ] `parseInt()`, `parseFloat()`
- [ ] Implement Object constructor
- [ ] Implement Object static methods
  - [ ] `Object.create()`
  - [ ] `Object.keys()`
  - [ ] `Object.getPrototypeOf()`
  - [ ] `Object.setPrototypeOf()`
  - [ ] `Object.defineProperty()`
  - [ ] `Object.defineProperties()`
  - [ ] `Object.getOwnPropertyNames()`
- [ ] Implement Object prototype methods
  - [ ] `toString()`
  - [ ] `valueOf()`
  - [ ] `hasOwnProperty()`
  - [ ] `isPrototypeOf()`
  - [ ] `propertyIsEnumerable()`
- [ ] Write unit tests

**Deliverable:** Global and Object built-ins.

#### Week 30: Function and Array Built-ins

**Tasks:**
- [ ] Implement Function constructor
- [ ] Implement Function prototype methods
  - [ ] `call()`
  - [ ] `apply()`
  - [ ] `bind()`
  - [ ] `toString()`
- [ ] Implement Array constructor
- [ ] Implement Array static methods
  - [ ] `Array.isArray()`
- [ ] Implement Array prototype methods (mutation)
  - [ ] `push()`, `pop()`
  - [ ] `shift()`, `unshift()`
  - [ ] `splice()`
  - [ ] `reverse()`
  - [ ] `sort()`
- [ ] Implement Array prototype methods (access)
  - [ ] `slice()`
  - [ ] `concat()`
  - [ ] `join()`
  - [ ] `indexOf()`, `lastIndexOf()`
- [ ] Write unit tests

**Deliverable:** Function and Array built-ins.

#### Week 31: Array Iteration and String Built-ins

**Tasks:**
- [ ] Implement Array iteration methods
  - [ ] `forEach()`
  - [ ] `map()`
  - [ ] `filter()`
  - [ ] `reduce()`, `reduceRight()`
  - [ ] `every()`, `some()`
- [ ] Implement String constructor
- [ ] Implement String prototype methods (access)
  - [ ] `charAt()`, `charCodeAt()`, `codePointAt()`
  - [ ] `slice()`, `substring()`, `substr()`
  - [ ] `indexOf()`, `lastIndexOf()`
  - [ ] `concat()`
- [ ] Implement String prototype methods (transform)
  - [ ] `toLowerCase()`, `toUpperCase()` (ASCII only)
  - [ ] `trim()`, `trimStart()`, `trimEnd()`
- [ ] Implement String prototype methods (regexp)
  - [ ] `match()`
  - [ ] `replace()`, `replaceAll()`
  - [ ] `search()`
  - [ ] `split()`
- [ ] Write unit tests

**Deliverable:** Array iteration and String built-ins.

#### Week 32: Number, Math, and Error Built-ins

**Tasks:**
- [ ] Implement Number constructor
- [ ] Implement Number static properties
  - [ ] `MAX_VALUE`, `MIN_VALUE`
  - [ ] `NaN`, `POSITIVE_INFINITY`, `NEGATIVE_INFINITY`
- [ ] Implement Number prototype methods
  - [ ] `toString()`
  - [ ] `toFixed()`
  - [ ] `toExponential()`
  - [ ] `toPrecision()`
- [ ] Implement Math object
- [ ] Implement Math constants
  - [ ] `PI`, `E`, `LN2`, `LN10`, etc.
- [ ] Implement Math functions
  - [ ] `abs()`, `floor()`, `ceil()`, `round()`
  - [ ] `min()`, `max()`
  - [ ] `pow()`, `sqrt()`
  - [ ] `sin()`, `cos()`, `tan()`, `atan()`, `atan2()`
  - [ ] `exp()`, `log()`
  - [ ] `random()`
- [ ] Implement Error constructors
  - [ ] Error, TypeError, ReferenceError, etc.
- [ ] Implement Error prototype methods
  - [ ] `toString()`
- [ ] Implement stack trace capture
- [ ] Write unit tests

**Deliverable:** Number, Math, and Error built-ins.

#### Week 33: JSON and RegExp Built-ins

**Tasks:**
- [ ] Implement JSON object
- [ ] Implement `JSON.parse()`
  - [ ] Lexer for JSON
  - [ ] Recursive descent parser
  - [ ] Reviver function support
- [ ] Implement `JSON.stringify()`
  - [ ] Serialization
  - [ ] Replacer function support
  - [ ] Space parameter
  - [ ] Handle circular references
- [ ] Implement RegExp constructor
- [ ] Implement basic regexp engine
  - [ ] Pattern compilation
  - [ ] Matching algorithm
  - [ ] Capture groups
  - [ ] Flags support
- [ ] Implement RegExp prototype methods
  - [ ] `test()`
  - [ ] `exec()`
- [ ] Write unit tests
  - [ ] Test JSON round-trips
  - [ ] Test regexp matching

**Deliverable:** JSON and RegExp built-ins.

#### Week 34: Typed Arrays and Finalization

**Tasks:**
- [ ] Implement ArrayBuffer constructor
- [ ] Implement typed array constructors
  - [ ] Int8Array, Uint8Array, Uint8ClampedArray
  - [ ] Int16Array, Uint16Array
  - [ ] Int32Array, Uint32Array
  - [ ] Float32Array, Float64Array
- [ ] Implement typed array prototype methods
  - [ ] `set()`, `subarray()`
  - [ ] Standard array methods where applicable
- [ ] Implement DataView (optional)
- [ ] Review all built-ins for completeness
- [ ] Cross-reference with ECMAScript spec
- [ ] Write compatibility tests
- [ ] Write comprehensive integration tests

**Deliverable:** Complete built-in library, 90%+ coverage.

### 8.8 Phase 8: Standard Library Build Tool (Weeks 35-36)

**Tasks:**
- [ ] Create `mquickjs-build` crate
- [ ] Implement stdlib parser
  - [ ] Reuse compiler from mquickjs
  - [ ] Parse JavaScript stdlib source
- [ ] Implement bytecode generation
  - [ ] Compile stdlib to bytecode
  - [ ] Relocate to address 0
- [ ] Implement Rust code emission
  - [ ] Emit const arrays
  - [ ] Emit atom table
  - [ ] Emit C function table
  - [ ] Emit class definitions
- [ ] Integrate with build process
  - [ ] Build script
  - [ ] Generate stdlib.rs
  - [ ] Include in main build
- [ ] Implement ROM relocation
- [ ] Test instantiation speed
- [ ] Optimize for size

**Deliverable:** Working build tool, ROM-resident stdlib.

### 8.9 Phase 9: Optimization and Polish (Weeks 37-40)

**Tasks:**
- [ ] Profile hot paths
  - [ ] VM dispatch
  - [ ] Property lookup
  - [ ] Arithmetic operations
  - [ ] GC
- [ ] Optimize VM dispatch
  - [ ] Consider function pointer table
  - [ ] Benchmark vs match
- [ ] Optimize GC
  - [ ] Reduce marking overhead
  - [ ] Optimize compaction
- [ ] Add SIMD optimizations (if applicable)
  - [ ] String operations
  - [ ] Array operations
- [ ] Reduce memory overhead
  - [ ] Review struct layouts
  - [ ] Minimize padding
  - [ ] Bitpack more fields
- [ ] Create comprehensive benchmarks
  - [ ] Fibonacci
  - [ ] Array operations
  - [ ] String operations
  - [ ] Object creation
  - [ ] Property access
- [ ] Compare to C version
  - [ ] Performance
  - [ ] Memory usage
- [ ] Memory profiling
  - [ ] Heap usage
  - [ ] Stack usage
- [ ] Stress testing
  - [ ] Long-running scripts
  - [ ] Deep recursion
  - [ ] Large allocations
- [ ] Documentation polish
  - [ ] API docs
  - [ ] Examples
  - [ ] Architecture guide
- [ ] API ergonomics review
  - [ ] Naming consistency
  - [ ] Error handling
  - [ ] Builder patterns
- [ ] Safety audit
  - [ ] Review all unsafe
  - [ ] MIRI testing
  - [ ] Fuzz testing
- [ ] Consider optional features
  - [ ] `no_std` support
  - [ ] Soft-float
  - [ ] Minimal stdlib

**Deliverable:** Production-ready, optimized implementation.

### 8.10 Phase 10: CLI and Tools (Weeks 41-42)

**Tasks:**
- [ ] Create `mqjs` crate
- [ ] Implement REPL
  - [ ] Read-eval-print loop
  - [ ] Multi-line input
  - [ ] Error handling
- [ ] Implement script execution
  - [ ] Load file
  - [ ] Execute
  - [ ] Print result
- [ ] Implement bytecode compilation
  - [ ] Compile to bytecode
  - [ ] Save to file
  - [ ] Load from file
- [ ] Implement memory limit control
  - [ ] Command-line option
  - [ ] Enforce limit
- [ ] Implement statistics dumping
  - [ ] Memory usage
  - [ ] Object counts
  - [ ] GC stats
- [ ] Integrate readline
  - [ ] History
  - [ ] Tab completion
  - [ ] Emacs bindings
- [ ] Implement command-line arguments
  - [ ] Argument parsing
  - [ ] Help text
  - [ ] Version info
- [ ] Write user documentation
  - [ ] Usage guide
  - [ ] Examples
  - [ ] API reference
- [ ] Create example programs
  - [ ] Hello world
  - [ ] Fibonacci
  - [ ] Tree traversal
  - [ ] Async patterns

**Deliverable:** Full-featured CLI with documentation.

## 9. Success Criteria

### 9.1 Functional Requirements

- [ ] Parse and execute valid ES5 strict mode JavaScript
- [ ] Pass 80%+ of relevant ECMAScript 5.1 test suite
- [ ] Support all MicroQuickJS features
- [ ] No memory leaks
- [ ] No undefined behavior
- [ ] Correct GC behavior

### 9.2 Performance Requirements

- [ ] Execution speed within 1.2x of C version
- [ ] Memory overhead < 20% vs C version
- [ ] GC pause time < 2ms for 64KB heap
- [ ] Minimum RAM: 12 kB (allowing 20% overhead)
- [ ] Parser throughput > 500 KB/s

### 9.3 Quality Requirements

- [ ] Test coverage 85%+ overall
- [ ] Test coverage 90%+ for memory management
- [ ] Zero unsafe code in public API
- [ ] All unsafe code documented with invariants
- [ ] Passes MIRI checks
- [ ] No crashes in 24-hour fuzz testing
- [ ] API documentation complete
- [ ] Architecture documentation complete

### 9.4 Usability Requirements

- [ ] Simple public API
- [ ] Good error messages
- [ ] Examples provided
- [ ] CLI is user-friendly
- [ ] Build process is straightforward

## 10. Milestone Schedule

### Milestone 1: MVP (End of Week 22)
**Deliverables:**
- Memory management with GC
- Value system
- Object system
- Bytecode definition
- Working VM

**Success Criteria:**
- Execute simple scripts (arithmetic, variables, functions)
- No built-ins yet
- 85% test coverage for implemented features

### Milestone 2: Complete Engine (End of Week 28)
**Deliverables:**
- Full parser/compiler
- All VM opcodes
- Exception handling

**Success Criteria:**
- Parse complex scripts
- Execute all language features
- Still no stdlib

### Milestone 3: Standard Library (End of Week 34)
**Deliverables:**
- All built-in objects
- All built-in methods
- Compliance testing

**Success Criteria:**
- Pass 60%+ of ES5 tests
- All documented features work

### Milestone 4: Production Ready (End of Week 40)
**Deliverables:**
- Optimized implementation
- Complete documentation
- Build tool
- CLI

**Success Criteria:**
- Pass 80%+ of ES5 tests
- Performance within 1.2x of C
- Production quality

### Milestone 5: Release 1.0 (End of Week 42)
**Deliverables:**
- Polished CLI
- User documentation
- Examples
- Announcement

**Success Criteria:**
- All success criteria met
- Ready for public use

## 11. Open Questions and Future Work

### 11.1 Design Decisions to Resolve

1. **GC Strategy:** Index-based vs pointer threading?
   - Recommendation: Start with index-based, profile, then decide

2. **VM Dispatch:** Match vs function pointers?
   - Recommendation: Start with match, optimize if needed

3. **Value Representation:** Newtype vs enum?
   - Recommendation: Newtype for performance, enum wrapper for ergonomics

4. **No_std Support:** Initial target or later?
   - Recommendation: Design for it, implement later

### 11.2 Future Enhancements (Post-1.0)

- ES6+ features (where feasible)
- Better regexp engine
- Debugger protocol
- Profiler
- JIT compilation (very long term)
- WASM backend
- Async/await (if minimal overhead)
- Module system
- Better Date implementation

### 11.3 Research Areas

- Zero-copy string interning
- Incremental GC
- Generational GC
- Inline caching for property access
- Type specialization
- SIMD optimizations

## 12. Resources and References

### 12.1 Documentation
- [MicroQuickJS Repository](https://github.com/bellard/mquickjs)
- [QuickJS Documentation](https://bellard.org/quickjs/)
- [ECMAScript 5.1 Specification](https://262.ecma-international.org/5.1/)
- Project documentation in `/root/rustmicroquickjs/notes/`

### 12.2 Rust Resources
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [The Rustonomicon (Unsafe Rust)](https://doc.rust-lang.org/nomicon/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### 12.3 Algorithms
- Deutsch-Schorr-Waite pointer threading
- Tri-color marking
- Exponential-Golomb coding
- Precedence climbing (expression parsing)

### 12.4 Inspirational Projects
- [boa](https://github.com/boa-dev/boa) - JavaScript engine in Rust
- [quickjs-rs](https://github.com/quickjs-rs/quickjs-rs) - QuickJS bindings
- [rquickjs](https://github.com/DelSkayn/rquickjs) - QuickJS bindings
- Note: These are bindings or different designs; we're doing a native port

## 13. Conclusion

This implementation plan provides a roadmap for porting MicroQuickJS from C to native Rust over 42 weeks. The plan emphasizes:

1. **Correctness first:** Extensive testing at every phase
2. **Safety where possible:** Minimize unsafe code, document all invariants
3. **Performance awareness:** Profile and optimize, but don't sacrifice correctness
4. **Incremental progress:** Clear milestones and deliverables
5. **Realistic scope:** Focus on MicroQuickJS feature set, no scope creep

The key technical challenges are:
- Implementing a safe compacting GC in Rust
- Maintaining C-level performance
- Keeping memory overhead minimal
- Handling complex pointer manipulation safely

Success will be measured by:
- Functional correctness (ECMAScript compliance)
- Performance (within 1.2x of C)
- Safety (no undefined behavior, high test coverage)
- Usability (good API, documentation, tools)

With careful execution of this plan, we can create a production-ready JavaScript engine in Rust that maintains MicroQuickJS's unique value proposition: running JavaScript in extremely constrained environments.

---

**Document Version:** 1.0
**Last Updated:** 2025-12-24
**Status:** Draft Implementation Plan
**Next Steps:** Review, refine, and begin Phase 0
