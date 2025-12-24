# MicroQuickJS Memory Management

## Overview

MicroQuickJS uses a custom memory management system designed for embedded systems with extreme constraints. The key innovation is combining a simple bump allocator with a tracing, compacting garbage collector.

## Memory Allocation Strategy

### User-Provided Buffer

All memory comes from a single contiguous buffer provided by the user:

```c
uint8_t mem_buf[8192];
JSContext *ctx = JS_NewContext(mem_buf, sizeof(mem_buf), &js_stdlib);
```

**Memory Layout:**
```
Low Address
┌─────────────────────┐ ← heap_base
│     Heap            │
│  (grows upward)     │
│                     │
├─────────────────────┤ ← heap_free (allocation point)
│                     │
│    Free Space       │
│                     │
├─────────────────────┤ ← stack_bottom (minimum safe position)
│                     │
│     VM Stack        │
│  (grows downward)   │
├─────────────────────┤ ← sp (stack pointer)
│                     │
├─────────────────────┤ ← stack_top
│   JSContext         │
│   + class_proto[]   │
└─────────────────────┘ ← heap_base + mem_size
High Address
```

### Bump Allocator

```c
static void *js_malloc(JSContext *ctx, uint32_t size, int mtag)
{
    JSMemBlockHeader *p;

    size = (size + JSW - 1) & ~(JSW - 1);  // Align to word size

    if (check_free_mem(ctx, ctx->stack_bottom, size))
        return NULL;  // Out of memory

    p = (JSMemBlockHeader *)ctx->heap_free;
    ctx->heap_free += size;  // Bump pointer

    p->mtag = mtag;
    p->gc_mark = 0;
    return p;
}
```

**Characteristics:**
- O(1) allocation
- No free list
- No fragmentation (until GC)
- Deterministic behavior
- Cache-friendly sequential allocation

### Free Operation

```c
static void js_free(JSContext *ctx, void *ptr)
{
    uint8_t *ptr1 = ptr;
    ptr1 += get_mblock_size(ptr1);
    if (ptr1 == ctx->heap_free)
        ctx->heap_free = ptr;  // Can only free last block
}
```

**Limitation:** Only the most recently allocated block can be freed. This is acceptable because most objects are freed by GC.

### Shrink Operation

```c
static void *js_shrink(JSContext *ctx, void *ptr, uint32_t new_size)
{
    uint32_t old_size = get_mblock_size(ptr);
    uint32_t diff = old_size - new_size;

    if (diff > 0)
        set_free_block((uint8_t *)ptr + new_size, diff);

    return ptr;
}
```

Creates a free block after the shrunk object. Free blocks are merged during GC.

## Garbage Collection

### When GC Runs

1. **Out of Memory:** When `heap_free` approaches `stack_bottom`
2. **Explicit:** User calls `JS_GC(ctx)`
3. **Before Critical Operations:** Parser, etc.

### GC Algorithm: Mark and Compact

MicroQuickJS uses a **tri-color marking** algorithm followed by **compaction with pointer threading**.

#### Phase 1: Mark (Trace Reachable Objects)

```c
void JS_GC(JSContext *ctx)
{
    gc_mark_all(ctx, FALSE);
    gc_compact_heap(ctx);
}
```

**Mark Process:**

1. **Initialize GC Stack:** Use free heap space as mark stack
2. **Mark Roots:**
   - Current exception
   - Global object
   - Class prototypes
   - VM stack values
   - GC references (JSGCRef)
   - Parser state (if compiling)
3. **Trace Graph:** Follow object references depth-first
4. **Handle Overflow:** If mark stack overflows, scan heap to resume

**Mark Stack:**
```c
static void gc_mark_all(JSContext *ctx, BOOL keep_atoms)
{
    GCMarkState s_s, *s = &s_s;

    s->ctx = ctx;
    s->overflow = FALSE;
    s->gs_top = ctx->sp;          // Use VM stack space
    s->gsp = s->gs_top;
    s->gs_bottom = (JSValue *)ctx->heap_free;  // Use free heap space

    // Mark all roots...
}
```

**Marking Strategy:**
- Small objects pushed on mark stack
- Large objects (JSValueArray) handled specially to save stack
- If stack overflows, set flag and continue
- On overflow, scan entire heap for marked-but-unscanned objects

**GC Mark Bit:**
Every heap block has a `gc_mark` bit in its header:
- 0 = white (not yet visited)
- 1 = black (visited and scanned)

#### Phase 2: Sweep and Compact

**Sweep:**
```c
// In gc_mark_all(), after marking:
uint8_t *ptr = ctx->heap_base;
while (ptr < ctx->heap_free) {
    JSFreeBlock *b = (JSFreeBlock *)ptr;
    if (b->gc_mark) {
        b->gc_mark = 0;  // Clear for next GC
    } else {
        // Call finalizers for user objects
        // Mark as free block
        set_free_block(b, size);
    }
}
```

**Compact:**

The compaction algorithm uses **pointer threading** (Deutsch-Schorr-Waite):

1. **Thread Pointers:**
   - For each live object, find all pointers to it
   - Replace each pointer with a link to the next pointer
   - Store the final value in the object's first field
   - Object now has a linked list of all references to it

2. **Move Objects:**
   - Move live objects down to eliminate gaps
   - Objects move to lower addresses

3. **Unthread and Update:**
   - Walk the threaded pointer chains
   - Update all pointers to new locations
   - Restore original values

```c
static void gc_compact_heap(JSContext *ctx)
{
    // Thread all pointers
    for (each live object)
        gc_thread_block(ctx, object);

    // Compact: move objects
    uint8_t *dst = ctx->heap_base;
    uint8_t *src = ctx->heap_base;
    while (src < ctx->heap_free) {
        if (is_live(src)) {
            gc_update_threaded_pointers(ctx, src, dst);
            if (dst != src)
                memmove(dst, src, size);
            dst += size;
        }
        src += size;
    }
    ctx->heap_free = dst;
}
```

**Threading Example:**
```
Before threading pointer to object X:
  pointer_location → X

After threading:
  X.first_field → old_value
  pointer_location → X.first_field

After second pointer to X:
  X.first_field → pointer_location (first)
  pointer_location2 → X.first_field

Thread chain: X → ptr2 → ptr1 → old_value
```

### Weak References

**Unique Strings Table:**
- Sorted array of interned strings
- Weak references (not roots)
- After marking, remove unmarked strings
- Compact remaining strings

**String Position Cache:**
- UTF-8 to UTF-16 position cache
- Weak references to strings
- Cleared if string not marked

### Finalizers

User classes can register finalizers:

```c
static const JSCFinalizer c_finalizer_table[] = {
    js_rectangle_finalizer,
    js_filled_rectangle_finalizer,
};
```

Called during sweep phase for unmarked objects:
```c
if (p->class_id >= JS_CLASS_USER && !p->gc_mark) {
    ctx->c_finalizer_table[p->class_id - JS_CLASS_USER](ctx, p->u.user.opaque);
}
```

**Important:** Finalizers cannot call JS code or allocate.

## GC Safe Programming

### The Pointer Invalidation Problem

**Problem:** GC can move objects, invalidating C pointers:

```c
// WRONG:
JSObject *obj = JS_VALUE_TO_PTR(some_value);
JSValue val = JS_NewObject(ctx);  // May trigger GC, obj is now invalid!
obj->proto = val;  // CRASH or corruption
```

### Solution: JSGCRef

Use `JSGCRef` to keep temporary references during allocations:

```c
// CORRECT:
JSGCRef obj_ref, val_ref;
JSValue *obj, *val;

obj = JS_PushGCRef(ctx, &obj_ref);
val = JS_PushGCRef(ctx, &val_ref);

*obj = some_value;
*val = JS_NewObject(ctx);  // GC may run, obj_ref.val updated automatically
JS_SetPropertyStr(ctx, *obj, "proto", *val);  // Safe

JS_PopGCRef(ctx, &val_ref);
JS_PopGCRef(ctx, &obj_ref);
```

**How It Works:**
- `JSGCRef` registers as GC root
- During compaction, GC updates `ref->val` to new address
- Always access through the pointer returned by `PushGCRef`

**Two Types of GC Refs:**

1. **Stack (LIFO):**
   ```c
   JSValue *JS_PushGCRef(JSContext *ctx, JSGCRef *ref);
   JSValue JS_PopGCRef(JSContext *ctx, JSGCRef *ref);
   ```
   - Fast
   - Must be popped in reverse order
   - For temporary locals

2. **List (any order):**
   ```c
   JSValue *JS_AddGCRef(JSContext *ctx, JSGCRef *ref);
   void JS_DeleteGCRef(JSContext *ctx, JSGCRef *ref);
   ```
   - Slower (linked list)
   - Can remove in any order
   - For long-lived temps

### Rules for C Code

1. **Never hold raw pointers across allocations**
   - Any JS API call might allocate
   - Use JSGCRef or reload pointer

2. **Minimize GC root count**
   - Only protect what you actively use
   - Pop/delete as soon as possible

3. **Stack discipline**
   - Push order matters with stack-based refs
   - Always pop in reverse order

4. **Read the source**
   - Look for `SAVE()` and `RESTORE()` macros
   - These mark allocation points

### SAVE/RESTORE Macros

Used internally to save/restore interpreter state:

```c
#define SAVE() do {                 \
    ctx->sp = sp;                   \
    ctx->fp = fp;                   \
} while (0)

#define RESTORE() do {              \
    sp = ctx->sp;                   \
    fp = ctx->fp;                   \
} while (0)
```

Called around any operation that might allocate and GC.

## Memory Debugging

### DEBUG_GC Mode

Enable with `#define DEBUG_GC`:

```c
#ifdef DEBUG_GC
static void *js_malloc(JSContext *ctx, uint32_t size, int mtag)
{
    // Force GC on every allocation
    JS_GC(ctx);
    // Modify allocated pointers
    return modify_pointer(real_malloc(ctx, size, mtag));
}
#endif
```

**Purpose:** Catch pointer invalidation bugs early
- Every allocation triggers full GC
- Pointers are scrambled
- Any use of stale pointer crashes immediately

### Memory Dumps

```c
void JS_DumpMemory(JSContext *ctx, JS_BOOL is_long);
```

Outputs:
- Heap usage
- Object counts by type
- Largest objects
- Stack usage
- Fragmentation info

## Performance Characteristics

### Time Complexity

- **Allocation:** O(1) - bump pointer
- **GC Mark:** O(live objects)
- **GC Sweep:** O(heap size)
- **GC Compact:** O(heap size + pointers)
- **Total GC:** O(heap size)

### Space Overhead

- **Per-block:** 4 bytes (32-bit) or 8 bytes (64-bit)
- **GC mark stack:** Uses free heap space
- **No free list:** All free space is contiguous

### GC Pause Time

- **Proportional to heap size**
- **Deterministic** - no generational collection
- **Predictable** - single-threaded, runs to completion
- **Typical:** ~1ms for 64KB heap on modern CPU

## ROM Support

### Read-Only Memory Detection

```c
#define JS_IS_ROM_PTR(ctx, ptr) \
    ((uintptr_t)(ptr) < (uintptr_t)ctx || \
     (uintptr_t)(ptr) >= (uintptr_t)ctx->stack_top)
```

ROM pointers are:
- Below heap_base, or
- Above stack_top

These are never marked, moved, or freed.

### Standard Library in ROM

Standard library compiled to const C arrays:

```c
static const JSWord js_stdlib_table[] = {
    // Precompiled atoms, objects, prototypes...
};
```

Instantiation just sets up pointers - no allocation needed.

### Bytecode in ROM

Compiled scripts can be relocated and flashed to ROM:

```c
JS_RelocateBytecode(ctx, buf, buf_len);  // Adjust pointers
JS_LoadBytecode(ctx, rom_buf);           // Load from ROM
```

Requires no RAM for bytecode storage.

## Rust Port Considerations

### Memory Safety Challenges

1. **Custom Allocator:**
   - Use `alloc::alloc::Allocator` trait?
   - Or custom arena allocator
   - Need stable addresses during non-GC operations

2. **Pointer Threading:**
   - Requires raw pointer manipulation
   - Unsafe code carefully isolated
   - Possibly use `UnsafeCell` for interior mutability

3. **GC Rooting:**
   - Translate JSGCRef to Rust handles
   - Lifetime tracking at type level?
   - Consider arena indices instead of pointers

4. **No RefCell:**
   - RefCell has runtime overhead
   - Consider explicit unsafe interior mutability
   - Document invariants clearly

### Possible Approaches

**Option 1: Generational Indices**
```rust
struct Handle(u32, PhantomData<Invariant>);
struct Arena {
    objects: Vec<Object>,
    free_list: Vec<u32>,
}
```
- Handles never invalidated by compaction
- Arena manages actual storage
- Trade-off: extra indirection

**Option 2: Unsafe Pointers with Epochs**
```rust
struct GcPtr<T> {
    ptr: *mut T,
    epoch: u32,
}
```
- Epoch incremented on GC
- Detect stale pointers
- Still requires unsafe

**Option 3: Arena with Cell**
```rust
struct Arena {
    memory: UnsafeCell<Vec<u8>>,
}
```
- Interior mutability for allocations
- Type-safe interface
- GC implementation in unsafe block

### Memory Layout

Rust structs need careful layout control:

```rust
#[repr(C)]
struct JSObject {
    header: MemBlockHeader,
    proto: JSValue,
    props: JSValue,
    class_specific: ObjectData,
}

#[repr(C)]
union ObjectData {
    closure: ClosureData,
    array: ArrayData,
    // ...
}
```

### Allocation API

```rust
impl Context {
    fn alloc<T>(&mut self, value: T) -> Result<Handle<T>, OutOfMemory> {
        // Check space
        // Possibly GC
        // Allocate
    }

    fn gc(&mut self) {
        // Mark
        // Sweep
        // Compact
    }
}
```

### Testing Strategy

- Unit tests for allocator
- Fuzz testing for GC
- Stress tests with random allocations
- Memory leak detection
- Valgrind/MIRI for unsafe code
