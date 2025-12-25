# MicroQuickJS Data Structures

## Core Types

### JSValue (mquickjs.h)

The fundamental value type in MQuickJS. Size is platform-dependent:
- 32 bits on 32-bit platforms
- 64 bits on 64-bit platforms

```c
typedef uint32_t JSValue;  // 32-bit
typedef uint64_t JSValue;  // 64-bit (JS_PTR64)
```

#### Value Encoding

Values use NaN-boxing-like technique with multiple tag types:

**Tags (low bits):**
- `JS_TAG_INT` (0): 31-bit signed integer (top bit unused, bit 0 = 0)
- `JS_TAG_PTR` (1): Pointer to heap block (aligned, bit 0 = 1)
- `JS_TAG_SPECIAL` (3): Special values using more bits
- `JS_TAG_SHORT_FLOAT` (5): Short float on 64-bit systems

**Special Tags (5 bits):**
- `JS_TAG_BOOL`: true/false
- `JS_TAG_NULL`: null
- `JS_TAG_UNDEFINED`: undefined
- `JS_TAG_EXCEPTION`: exception marker
- `JS_TAG_SHORT_FUNC`: C function index (no heap allocation)
- `JS_TAG_UNINITIALIZED`: uninitialized variable marker
- `JS_TAG_STRING_CHAR`: Single unicode codepoint (1-2 UTF-16 code units)
- `JS_TAG_CATCH_OFFSET`: Used in exception handling

**Value Extraction Macros:**
```c
#define JS_VALUE_GET_INT(v) ((int)(v) >> 1)
#define JS_VALUE_TO_PTR(v) ((void *)((uintptr_t)(v) - 1))
#define JS_VALUE_FROM_PTR(ptr) ((JSWord)((uintptr_t)(ptr) + 1))
```

### Memory Block Header (mquickjs.c)

Every heap-allocated block starts with this header:

```c
typedef struct {
    JSWord gc_mark: 1;      // GC mark bit
    JSWord mtag: 3;         // Memory tag (type of block)
    // Remaining bits available for subtype use
} JSMemBlockHeader;
```

**Memory Tags (mtag):**
- `JS_MTAG_FREE`: Free block
- `JS_MTAG_OBJECT`: JavaScript object
- `JS_MTAG_FLOAT64`: 64-bit floating point number
- `JS_MTAG_STRING`: String
- `JS_MTAG_FUNCTION_BYTECODE`: Function bytecode
- `JS_MTAG_VALUE_ARRAY`: Array of JSValues
- `JS_MTAG_BYTE_ARRAY`: Array of bytes
- `JS_MTAG_VARREF`: Variable reference (closure)

### JSString (mquickjs.c)

Strings are stored in UTF-8 encoding:

```c
typedef struct {
    JS_MB_HEADER;
    JSWord is_unique: 1;    // In unique string table
    JSWord is_ascii: 1;     // Pure ASCII (optimization)
    JSWord is_numeric: 1;   // Represents a number
    JSWord len: (32 - JS_MTAG_BITS - 3);  // UTF-8 byte length
    uint8_t buf[];          // UTF-8 data
} JSString;
```

**Key Properties:**
- UTF-8 encoding saves memory for ASCII-heavy code
- `is_ascii` flag enables fast path for many operations
- `is_unique` indicates string is in atom table (interned)
- `is_numeric` helps with fast property key lookup
- Maximum length: ~512 MB on 32-bit, much larger on 64-bit

### JSObject (mquickjs.c)

The base object structure:

```c
struct JSObject {
    JS_MB_HEADER;
    JSWord class_id: 8;           // Object class
    JSWord extra_size: (32 - JS_MTAG_BITS - 8);  // Additional JSValues

    JSValue proto;                // Prototype object or JS_NULL
    JSValue props;                // Property table (JSValueArray)

    union {
        JSClosureData closure;
        JSCFunctionData cfunc;
        JSArrayData array;
        JSErrorData error;
        JSArrayBuffer array_buffer;
        JSTypedArray typed_array;
        JSRegExp regexp;
        JSObjectUserData user;
    } u;
};
```

**Minimum Size:** 12 bytes on 32-bit (3 words: header + proto + props)

**extra_size:** Number of additional JSValue fields in the union

### Property System

Properties stored in a hash table structure:

```c
typedef struct {
    JSValue key;              // Property name (string or int)
    JSValue value;            // Property value
    uint32_t hash_next : 30;  // Next in hash chain
    uint32_t prop_type : 2;   // Property type
} JSProperty;
```

**Property Types:**
- `JS_PROP_NORMAL`: Regular property
- `JS_PROP_GETSET`: Getter/setter (value is 2-element array)
- `JS_PROP_VARREF`: Variable reference (for globals)
- `JS_PROP_SPECIAL`: ROM properties (prototype, constructor)

**Property Storage Format (JSValueArray):**
```
[header]
[prop_count]    // Number of active properties
[hash_mask]     // hash_size - 1
[hash_table]    // Array of indices (hash_size elements)
[properties]    // Array of JSProperty structures
```

### JSValueArray (mquickjs.c)

Dynamic array of JSValues:

```c
typedef struct {
    JS_MB_HEADER;
    JSWord size: (32 - JS_MTAG_BITS);
    JSValue arr[];
} JSValueArray;
```

Used for:
- Property tables
- Array storage
- Constant pools
- Atom tables

### JSByteArray (mquickjs.c)

Dynamic array of bytes:

```c
typedef struct {
    JS_MB_HEADER;
    JSWord size: (32 - JS_MTAG_BITS);
    uint8_t buf[];
} JSByteArray;
```

Used for:
- Bytecode storage
- Regexp bytecode
- Debug info (pc2line)

### JSFunctionBytecode (mquickjs.c)

Compiled function information:

```c
typedef struct JSFunctionBytecode {
    JS_MB_HEADER;
    JSWord has_arguments : 1;
    JSWord has_local_func_name : 1;
    JSWord has_column : 1;
    JSWord arg_count : 16;
    // Padding bits...

    JSValue func_name;    // Function name or JS_NULL
    JSValue byte_code;    // JSByteArray of bytecode
    JSValue cpool;        // Constant pool (JSValueArray)
    JSValue vars;         // Variable names (debug)
    JSValue ext_vars;     // External variable info
    uint16_t stack_size;  // Max stack depth
    uint16_t ext_vars_len;
    JSValue filename;     // Source file name
    JSValue pc2line;      // Line number mapping
    uint32_t source_pos;  // Source position
} JSFunctionBytecode;
```

### Object Type Unions

#### JSClosureData
```c
typedef struct {
    JSValue func_bytecode;  // JSFunctionBytecode
    JSValue var_refs[];     // Captured variables (JSVarRef array)
} JSClosureData;
```

#### JSArrayData
```c
typedef struct {
    JSValue tab;      // JS_NULL or JSValueArray
    uint32_t len;     // Logical length (≤ 2^30-1)
} JSArrayData;
```

#### JSCFunctionData
```c
typedef struct {
    uint32_t idx;        // Index into c_function_table
    JSValue params;      // Optional parameters
} JSCFunctionData;
```

### JSVarRef (mquickjs.c)

Variable reference for closures:

```c
typedef struct JSVarRef {
    JS_MB_HEADER;
    JSWord is_detached : 1;
    // Padding...
    union {
        JSValue value;              // Detached: direct value
        struct {
            JSValue next;           // Linked list of refs
            JSValue *pvalue;        // Attached: pointer to value
        };
    } u;
} JSVarRef;
```

**Lifecycle:**
- Attached: Points to stack variable
- Detached: Parent function exited, holds value directly

### JSContext (mquickjs.c)

The main execution context:

```c
struct JSContext {
    // Memory management
    uint8_t *heap_base;
    uint8_t *heap_free;      // First free byte in heap
    uint8_t *stack_top;      // Stack limit
    JSValue *stack_bottom;   // Minimum safe stack position
    JSValue *sp;             // Current stack pointer
    JSValue *fp;             // Frame pointer
    uint32_t min_free_size;

    // State flags
    BOOL in_out_of_memory : 8;
    uint8_t n_rom_atom_tables;
    uint8_t string_pos_cache_counter;
    uint16_t class_count;
    int16_t interrupt_counter;
    BOOL current_exception_is_uncatchable : 8;

    // Parser state
    struct JSParseState *parse_state;
    int unique_strings_len;
    int js_call_rec_count;

    // GC roots
    JSGCRef *top_gc_ref;     // Stack-based GC refs
    JSGCRef *last_gc_ref;    // List-based GC refs

    // ROM data
    const JSWord *atom_table;
    const JSValueArray *rom_atom_tables[N_ROM_ATOM_TABLES_MAX];
    const JSCFunctionDef *c_function_table;
    const JSCFinalizer *c_finalizer_table;

    // Runtime state
    uint64_t random_state;
    JSInterruptHandler *interrupt_handler;
    JSWriteFunc *write_func;
    void *opaque;
    JSValue *class_obj;
    JSStringPosCacheEntry string_pos_cache[2];

    // GC roots (JSValue fields)
    JSValue unique_strings;
    JSValue current_exception;
    JSValue empty_props;
    JSValue global_obj;
    JSValue minus_zero;
    JSValue class_proto[];   // Variable length: 2 * class_count
};
```

### JSGCRef (mquickjs.h)

Temporary GC root for C code:

```c
typedef struct JSGCRef {
    JSValue val;
    struct JSGCRef *prev;  // Stack or list linkage
} JSGCRef;
```

**Usage Pattern:**
```c
JSGCRef obj_ref;
JSValue *obj = JS_PushGCRef(ctx, &obj_ref);
*obj = JS_NewObject(ctx);  // obj can move during allocations
// Use *obj for all operations
JS_PopGCRef(ctx, &obj_ref);
```

### Stack Frame Layout

When a function is called, a frame is pushed on the VM stack:

```
[FRAME_OFFSET_ARG0 + argc - 1]  ← arguments (highest address)
...
[FRAME_OFFSET_ARG0]
[FRAME_OFFSET_FUNC_OBJ]         ← function object
[FRAME_OFFSET_THIS]             ← 'this' value
[FRAME_OFFSET_CUR_PC]           ← saved PC
[FRAME_OFFSET_CALL_FLAGS]       ← call flags (argc, constructor bit)
[FRAME_OFFSET_PREV_FP]          ← previous frame pointer (encoded)
← fp points here (frame base)
[local variables...]
[stack temporaries...]
← sp points here (stack top)
```

**Frame Offsets:**
```c
#define FRAME_OFFSET_PREV_FP     0
#define FRAME_OFFSET_CALL_FLAGS  1
#define FRAME_OFFSET_CUR_PC      2
#define FRAME_OFFSET_THIS        3
#define FRAME_OFFSET_FUNC_OBJ    4
#define FRAME_OFFSET_ARG0        5
```

### Parse State (mquickjs.c)

State for the parser (single instance during compilation):

```c
typedef struct JSParseState {
    JSContext *ctx;
    JSToken token;

    BOOL got_lf : 8;
    BOOL is_eval : 8;
    BOOL has_retval : 8;
    BOOL is_repl : 8;
    BOOL has_column : 8;
    BOOL dropped_result : 8;

    JSValue source_str;
    JSValue filename_str;
    const uint8_t *source_buf;
    uint32_t buf_pos;
    uint32_t buf_len;

    JSValue cur_func;          // Current function bytecode
    JSValue byte_code;         // Bytecode being generated
    uint32_t byte_code_len;
    int last_opcode_pos;
    // ... more state for code generation
} JSParseState;
```

## Important Patterns for Rust Port

### Tagged Pointers
- C uses pointer tagging extensively (bottom bits)
- Rust needs custom pointer types or newtype wrappers
- Consider using NonNull with phantom data

### Bit Fields
- C uses bit fields in structs heavily
- Rust should use explicit bit manipulation or crates like `bitfield`

### Flexible Array Members
- C uses `arr[]` for variable-length trailing data
- Rust needs DSTs or separate allocation strategy

### Union Discriminants
- C relies on class_id or mtag for union discrimination
- Rust enums provide type safety but may have size overhead
- Consider `#[repr(C)]` unions with manual discrimination

### Interior Mutability
- GC modifies objects through const pointers
- Rust needs UnsafeCell or RefCell patterns
- Arena-based allocation may help

### Pointer Stability
- C code assumes pointers become invalid across allocations
- Rust should use handles/indices into arena
- Or unsafe code with careful documentation
