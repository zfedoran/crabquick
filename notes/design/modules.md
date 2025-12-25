# MicroQuickJS Modules and Components

## Main Source Files

### mquickjs.c (~18,000 lines)

The monolithic main implementation file containing:

#### 1. Memory Management (lines ~500-600, ~11,780-12,700)

**Key Functions:**
- `js_malloc()`: Bump allocator, increments heap_free
- `js_mallocz()`: Zero-initialized allocation
- `js_free()`: Only frees last allocated block
- `js_shrink()`: Reduce block size, create free block
- `check_free_mem()`: Verify space before allocation
- `JS_GC()`: Main garbage collection entry point
- `gc_mark()`, `gc_mark_flush()`: GC marking phase
- `gc_mark_all()`: Mark all reachable objects
- `gc_thread_block()`: Thread pointers during compaction
- `gc_compact_heap()`: Move objects to eliminate gaps

**Memory Strategy:**
- No malloc/free dependency
- Everything in user-provided buffer
- Bump allocation until memory runs low
- Full GC with compaction when needed
- Simple and deterministic

#### 2. Value Operations (lines ~970-2300)

**Floating Point:**
- `js_alloc_float64()`: Allocate 64-bit float
- `JS_NewFloat64()`, `JS_NewInt32()`, `JS_NewInt64()`
- Short float encoding on 64-bit platforms
- Soft-float support for FPU-less systems

**Strings:**
- `js_alloc_string()`: Allocate UTF-8 string
- `JS_NewString()`, `JS_NewStringLen()`
- `JS_ToCString()`, `JS_ToCStringLen()`: String conversion
- `js_new_string_char()`: Single character string
- String interning (unique strings)
- UTF-8 ↔ UTF-16 position conversion

**Arrays:**
- `js_alloc_value_array()`: Allocate JSValue array
- `js_resize_value_array()`: Grow/shrink array
- `js_alloc_byte_array()`: Allocate byte array
- `js_resize_byte_array()`: Resize byte array

#### 3. Property System (lines ~2,400-3,300)

**Hash Table Implementation:**
- `js_alloc_props()`: Create property hash table
- `find_own_property()`: Hash-based property lookup
- `add_property()`: Insert new property
- `delete_property()`: Remove property
- `js_rehash_props()`: Rebuild hash table (when needed)

**Property Access:**
- `JS_GetPropertyInternal()`: Get property value
- `JS_SetPropertyInternal()`: Set property value
- `JS_GetPropertyStr()`, `JS_GetPropertyUint32()`
- `JS_DefinePropertyValue()`: Define new property
- `JS_DefinePropertyGetSet()`: Define getter/setter

**Fast Paths:**
- Integer array indices
- String property keys
- Direct prototype chain walk

#### 4. Object System (lines ~3,300-4,100)

**Object Creation:**
- `JS_NewObject()`: Plain object
- `JS_NewArray()`: Array object
- `JS_NewObjectClassUser()`: User class objects
- `js_create_from_ctor()`: Constructor pattern

**Prototypes:**
- `js_set_prototype_internal()`: Set __proto__
- `JS_GetPrototype()`: Get prototype
- Prototype chain traversal

**Special Objects:**
- Functions (closures, C functions)
- Arrays with special length handling
- Errors with stack traces
- Typed arrays
- RegExp objects

#### 5. Bytecode Interpreter (lines ~4,700-7,200)

**Main Loop:**
- `JS_Call()`: Execute bytecode
- Computed goto dispatch (if available)
- Switch-based dispatch (fallback)

**Opcode Categories:**

*Stack Operations:*
- `OP_drop`, `OP_dup`, `OP_swap`, `OP_nip`
- `OP_insert2`, `OP_insert3`, `OP_perm3`, `OP_rot3l`

*Value Pushing:*
- `OP_push_value`: Push constant
- `OP_push_i8`, `OP_push_i16`: Push integer
- `OP_push_const8`, `OP_push_const16`: Push from constant pool
- `OP_undefined`, `OP_null`, `OP_push_true`, `OP_push_false`

*Variable Access:*
- `OP_get_loc`, `OP_put_loc`: Local variables
- `OP_get_arg`, `OP_put_arg`: Arguments
- `OP_get_var_ref`, `OP_put_var_ref`: Closure variables

*Property Access:*
- `OP_get_field`, `OP_put_field`: Named properties
- `OP_get_array_el`, `OP_put_array_el`: Array elements
- `OP_get_length`: Fast array.length
- `OP_define_field`: Object literal properties

*Arithmetic:*
- `OP_add`, `OP_sub`, `OP_mul`, `OP_div`, `OP_mod`
- `OP_inc`, `OP_dec`, `OP_neg`, `OP_plus`
- `OP_shl`, `OP_sar`, `OP_shr`
- `OP_pow`: Exponentiation
- Fast paths for integers and short floats

*Comparison:*
- `OP_lt`, `OP_lte`, `OP_gt`, `OP_gte`
- `OP_eq`, `OP_neq`, `OP_strict_eq`, `OP_strict_neq`
- `OP_instanceof`, `OP_in`

*Logical:*
- `OP_not`, `OP_lnot`
- `OP_and`, `OP_or`, `OP_xor`

*Control Flow:*
- `OP_if_false`, `OP_if_true`: Conditional jumps
- `OP_goto`: Unconditional jump
- `OP_return`, `OP_return_undef`
- `OP_throw`: Throw exception
- `OP_catch`: Exception handler setup
- `OP_gosub`, `OP_ret`: Finally block execution

*Function Calls:*
- `OP_call`: Regular call
- `OP_call_constructor`: Constructor call
- `OP_call_method`: Method call
- Tail call optimization

*Iterators:*
- `OP_for_in_start`, `OP_for_of_start`
- `OP_for_of_next`: Iterator next

*Other:*
- `OP_fclosure`: Create closure
- `OP_typeof`: typeof operator
- `OP_delete`: delete operator
- `OP_regexp`: Create RegExp
- `OP_array_from`: Create array from elements

#### 6. Parser/Compiler (lines ~7,200-11,700)

**Lexer:**
- `next_token()`: Tokenize source
- Unicode support
- String literal parsing with escapes
- Number literal parsing
- Regexp literal parsing
- Template literal support
- Comment handling

**Parser:**
- Recursive descent without recursion (state stack)
- Expression parsing with precedence climbing
- Statement parsing
- Function parsing
- Class parsing (limited)

**Code Generation:**
- Direct bytecode emission
- Constant pool management
- Label management for jumps
- Local variable allocation
- Closure variable capture
- Peephole optimization
- Dead code elimination

**Debug Info:**
- Line number tracking
- Column number tracking (optional)
- Exponential-Golomb encoding for compression
- Source position mapping

#### 7. Built-in Functions (scattered throughout)

**Global Functions:**
- `eval()`, `isNaN()`, `isFinite()`
- `parseInt()`, `parseFloat()`

**Object Methods:**
- `Object.create()`, `Object.keys()`
- `Object.getPrototypeOf()`, `Object.setPrototypeOf()`
- `Object.defineProperty()`
- `toString()`, `hasOwnProperty()`

**Function Methods:**
- `call()`, `apply()`, `bind()`
- `toString()`

**Array Methods:**
- `push()`, `pop()`, `shift()`, `unshift()`
- `slice()`, `splice()`, `concat()`
- `join()`, `reverse()`, `sort()`
- `indexOf()`, `lastIndexOf()`
- `every()`, `some()`, `forEach()`, `map()`, `filter()`
- `reduce()`, `reduceRight()`

**String Methods:**
- `charAt()`, `charCodeAt()`, `codePointAt()`
- `slice()`, `substring()`, `substr()`
- `indexOf()`, `lastIndexOf()`
- `toLowerCase()`, `toUpperCase()` (ASCII only)
- `trim()`, `trimStart()`, `trimEnd()`
- `match()`, `replace()`, `replaceAll()`, `search()`, `split()`
- `concat()`

**Number Methods:**
- `toString()`, `toFixed()`, `toExponential()`, `toPrecision()`

**Math Functions:**
- Standard math functions (sin, cos, tan, etc.)
- `min()`, `max()`, `abs()`, `floor()`, `ceil()`, `round()`
- `pow()`, `sqrt()`, `exp()`, `log()`
- `random()` (simple PRNG)

**RegExp:**
- Pattern compilation
- Matching engine
- Capture groups
- Flags: i, m, s, y, u

**JSON:**
- `JSON.parse()`
- `JSON.stringify()`

**Typed Arrays:**
- ArrayBuffer
- Int8Array, Uint8Array, Uint8ClampedArray
- Int16Array, Uint16Array
- Int32Array, Uint32Array
- Float32Array, Float64Array

**Error Objects:**
- Error, EvalError, RangeError, ReferenceError
- SyntaxError, TypeError, URIError, InternalError

### mquickjs.h (382 lines)

Public API header defining:

**Types:**
- `JSContext`: Opaque context handle
- `JSValue`: Value representation
- `JSGCRef`: GC reference
- `JSCFunction`: C function signature
- `JSCFinalizer`: Finalizer callback
- `JSSTDLibraryDef`: Standard library definition

**Constants:**
- Value tags and special values
- Class IDs
- Eval flags
- Call flags

**Functions:**
- Context management
- Value creation
- Type checking
- Property access
- String operations
- Parsing and execution
- GC control
- Debugging

### mquickjs_priv.h (264 lines)

Private definitions for internal use:

**Constants:**
- Memory tags
- Property types
- Special values for internal use

**Runtime Helpers:**
- Function prototypes for all built-in methods
- Magic constants for multi-purpose functions

### mquickjs_opcode.h (264 lines)

Bytecode opcode definitions:

**Format Definitions:**
- Instruction formats (operand types)
- Operand encoding schemes

**Opcode Definitions:**
- All VM instructions
- Size, stack effect, format for each
- RegExp bytecode opcodes

## Supporting Files

### dtoa.c/h (1,620 lines)

David Gay's double-to-ASCII conversion:
- `js_dtoa()`: Double to string
- `js_strtod()`: String to double
- Precise decimal conversion
- Multiple conversion modes

### libm.c/h (2,260 lines)

Tiny math library:
- Standard math functions (sin, cos, tan, atan, atan2)
- Exponential/logarithm (exp, log, log2, log10)
- Power/root (pow, sqrt, cbrt)
- Hyperbolic functions
- Special values (NaN, Inf) handling
- Soft-float support (optional)

**softfp_template.h:**
- Software floating-point emulation
- For systems without hardware FPU
- IEEE 754 compliance

### cutils.c/h (178 + 355 lines)

Utility functions:
- String utilities (pstrcpy, pstrcat, strstart)
- Number parsing (strtoul, strtod)
- Integer to string conversion
- Unicode utilities
- Min/max helpers
- Bit manipulation (clz, ctz)

### list.h (99 lines)

Intrusive linked list macros (Linux kernel style):
- `init_list_head()`, `list_add()`, `list_del()`
- `list_for_each()`, `list_for_each_safe()`
- Used internally, not in public API

### mquickjs_build.c/h (932 + 97 lines)

Build-time tool to compile standard library:

**Purpose:**
- Parse JavaScript standard library
- Generate C data structures
- Emit ROM-able C code

**Process:**
1. Parse JS source
2. Generate bytecode
3. Relocate to address 0
4. Emit as const C arrays
5. Include in main build

**Output:**
- Atom table (interned strings)
- C function table
- Class definitions
- Global object structure

### mqjs.c (764 lines)

REPL and command-line interface:

**Features:**
- Interactive mode with readline
- Script execution
- Bytecode compilation and loading
- Memory limit control
- Dump statistics

**Functions:**
- `load_file()`: Load script from file
- `eval_buf()`: Evaluate buffer
- Command-line parsing
- REPL loop

### readline.c/readline_tty.c (742 + 246 lines)

Line editing support:
- History management
- Tab completion
- Emacs-style key bindings
- Terminal control
- Platform-specific TTY handling

## Module Dependencies

```
mquickjs.c
  ├── mquickjs.h (public API)
  ├── mquickjs_priv.h (internals)
  ├── mquickjs_opcode.h (bytecode)
  ├── mquickjs_atom.h (generated atoms)
  ├── cutils.h (utilities)
  ├── dtoa.h (number conversion)
  ├── libm.h (math)
  └── list.h (linked lists)

mqjs.c
  ├── mquickjs.h
  ├── mqjs_stdlib.h (generated stdlib)
  ├── readline.h
  └── cutils.h

example.c
  ├── mquickjs.h
  ├── example_stdlib.h (generated)
  └── cutils.h
```

## Build Process

1. Compile `mquickjs_build.c` (host compiler)
2. Run tool on `mqjs_stdlib.c` → generate `mqjs_stdlib.h` and `mquickjs_atom.h`
3. Compile main sources (target compiler)
4. Link final executable

## Code Organization Principles

**Monolithic vs Modular:**
- Single large .c file reduces header dependencies
- Easier to optimize (LTO not needed)
- Internal functions can be static
- Trade-off: harder to navigate

**Data-Oriented:**
- Flat memory layout
- Cache-friendly access patterns
- Minimal indirection

**Embedded-Friendly:**
- No dynamic library dependencies
- No thread-local storage
- No C++ exceptions
- Minimal stack usage

## Rust Port Module Structure

Suggested organization for Rust port:

```
src/
  lib.rs           - Public API
  context.rs       - JSContext
  value.rs         - JSValue and type checking
  memory.rs        - Allocator and GC
  object.rs        - Object system
  property.rs      - Property management
  string.rs        - String handling
  array.rs         - Array support
  function.rs      - Function objects
  bytecode/
    mod.rs         - Bytecode definitions
    compiler.rs    - Parser/compiler
    interpreter.rs - VM interpreter
    opcodes.rs     - Opcode implementations
  builtins/
    mod.rs         - Built-in registration
    object.rs      - Object methods
    array.rs       - Array methods
    string.rs      - String methods
    math.rs        - Math functions
    ... (one file per category)
  runtime/
    mod.rs         - Runtime support
    conversion.rs  - Type conversions
    operators.rs   - Operator implementations
    error.rs       - Error handling
  util/
    dtoa.rs        - Number formatting
    utf8.rs        - UTF-8 utilities
```
