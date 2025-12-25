# MicroQuickJS Execution Model

## Overview

MicroQuickJS executes JavaScript through a three-stage process:
1. **Parsing**: Source text → Bytecode
2. **Closure Creation**: Bytecode → Executable function
3. **Execution**: Bytecode interpretation in custom VM

## Parsing and Compilation

### Entry Points

```c
// Parse source to bytecode
JSValue JS_Parse(JSContext *ctx, const char *input, size_t input_len,
                 const char *filename, int eval_flags);

// Parse and execute
JSValue JS_Eval(JSContext *ctx, const char *input, size_t input_len,
                const char *filename, int eval_flags);
```

**Eval Flags:**
- `JS_EVAL_RETVAL`: Return last value instead of undefined
- `JS_EVAL_REPL`: Implicitly define global variables
- `JS_EVAL_STRIP_COL`: Strip column debug info (save memory)
- `JS_EVAL_JSON`: Parse as JSON
- `JS_EVAL_REGEXP`: Parse as regexp pattern

### Parser Architecture

**Non-Recursive Design:**
The parser avoids C recursion to minimize stack usage:

```c
typedef struct JSParseState {
    JSContext *ctx;
    JSToken token;          // Current token

    // Flags
    BOOL got_lf : 8;
    BOOL is_eval : 8;
    BOOL has_retval : 8;
    BOOL is_repl : 8;
    BOOL has_column : 8;

    // Source
    JSValue source_str;
    const uint8_t *source_buf;
    uint32_t buf_pos;
    uint32_t buf_len;

    // Current function being compiled
    JSValue cur_func;       // JSFunctionBytecode
    JSValue byte_code;      // JSByteArray
    // ... more state
} JSParseState;
```

**State Machine Pattern:**
Instead of recursive calls, parser uses:
- Explicit state tracking
- Loop-based iteration
- Stack stored in parse state

### Lexical Analysis

**Token Structure:**
```c
typedef struct JSToken {
    int val;           // Token type (TOK_*)
    JSValue value;     // Associated value (string, number, etc.)
    uint32_t line_num;
    uint32_t col_num;
} JSToken;
```

**Token Types:**
- Keywords: `if`, `while`, `function`, `return`, etc.
- Operators: `+`, `-`, `*`, `/`, `&&`, `||`, etc.
- Literals: numbers, strings, regexps, templates
- Identifiers and reserved words

**Lexer Features:**
- Unicode support (UTF-8 source)
- String escapes (`\n`, `\uXXXX`, `\u{XXXX}`)
- Numeric literals (decimal, hex, binary, octal)
- Template literals (partial support)
- Regexp literals with flags
- Automatic semicolon insertion

### Parsing Strategy

**One-Pass Compilation:**
- No AST construction
- Direct bytecode emission
- Constant pool for literals
- Forward references resolved with backpatching

**Expression Parsing:**
Uses precedence climbing for binary operators:

```c
static int js_parse_expr_binary(JSParseState *s, int state, int parse_flags)
{
    // Parse operand
    // While next token is binary op with higher precedence:
    //   Parse right operand
    //   Emit operator bytecode
}
```

**Statement Parsing:**
Recursive descent pattern (via state machine):
- Block statements
- If/else
- While/do-while/for loops
- Switch statements
- Try/catch/finally
- Function declarations
- Variable declarations

### Bytecode Generation

**Emission Functions:**
```c
static void emit_op(JSParseState *s, int op);
static void emit_u8(JSParseState *s, uint8_t val);
static void emit_u16(JSParseState *s, uint16_t val);
static void emit_u32(JSParseState *s, uint32_t val);
```

**Constant Pool:**
Literals stored in constant pool, referenced by index:

```c
static int add_const(JSParseState *s, JSValue val)
{
    // Add to cpool if not present
    // Return index
}
```

**Label Management:**
Forward jumps use labels that are backpatched:

```c
static int new_label(JSParseState *s);
static void label_here(JSParseState *s, int label);
static void emit_goto(JSParseState *s, int op, int label);
```

**Optimization Techniques:**

1. **Peephole Optimization:**
   - Remove redundant operations
   - Combine operations (e.g., `push + pop`)
   - Constant folding for literals

2. **Short Opcodes:**
   - Special opcodes for common cases
   - `push_0` through `push_7` for small integers
   - `get_loc0` through `get_loc3` for first locals

3. **Dead Code Elimination:**
   - Track reachability after `return`, `throw`
   - Skip code generation for unreachable code

### Variable Management

**Scope Types:**
- Global scope
- Function scope
- Block scope (for `let`, `const`)

**Variable Storage:**
- Local variables: In function stack frame
- Arguments: In caller's stack frame
- Closure variables: Captured in JSVarRef

**Closure Compilation:**
When function references outer variable:
1. Mark variable as captured
2. Create JSVarRef in outer function
3. Store varref index in closure
4. Access via `OP_get_var_ref`/`OP_put_var_ref`

### Debug Information

**Source Position Tracking:**
```c
typedef struct {
    uint32_t line_num;
    uint32_t col_num;
} JSSourcePos;
```

**PC-to-Line Mapping:**
Compressed using exponential-Golomb coding:

```c
// Encode delta from last position
// Small deltas use fewer bits
// Store in pc2line byte array
```

Compression ratios: 2-4× smaller than raw mapping.

**Column Info (Optional):**
Can be stripped with `JS_EVAL_STRIP_COL` to save memory.

### Function Output

Parser produces `JSFunctionBytecode`:

```c
typedef struct JSFunctionBytecode {
    JSValue func_name;
    JSValue byte_code;      // Executable bytecode
    JSValue cpool;          // Constant pool
    JSValue vars;           // Variable names (debug)
    JSValue ext_vars;       // Closure variables
    uint16_t stack_size;    // Max stack depth
    uint16_t arg_count;
    JSValue filename;
    JSValue pc2line;        // Debug info
} JSFunctionBytecode;
```

## Bytecode Format

### Instruction Encoding

**Opcode:** 1 byte
**Operands:** 0-4 bytes depending on instruction

**Common Formats:**
- `none`: No operands (e.g., `add`, `drop`)
- `u8`: 1-byte unsigned operand
- `i8`: 1-byte signed operand
- `u16`: 2-byte unsigned operand
- `i16`: 2-byte signed operand
- `u32`: 4-byte unsigned operand
- `label`: 4-byte jump target (PC offset)
- `const16`: 2-byte constant pool index

**Examples:**
```
push_i8 42           → [OP_push_i8] [42]
get_field "x"        → [OP_get_field] [idx_low] [idx_high]
if_false label       → [OP_if_false] [offset32]
```

### Stack-Based Execution

**Operand Stack:**
Bytecode operates on a value stack:

```
Stack grows down (toward lower addresses)
sp → [top value]
     [value 2]
     [value 3]
     ...
```

**Example: 1 + 2**
```
push_i8 1       sp: [1]
push_i8 2       sp: [2, 1]
add             sp: [3]
```

### Atom Table

**Indirect String References:**
String constants referenced via atom table:

```c
JSValue cpool[n];  // Constant pool
cpool[i] = "property_name"  // String value

OP_get_field i     // Get property at cpool[i]
```

**Advantages:**
- Atoms shared across all code
- Smaller bytecode
- Fast string comparison (pointer equality)

## Virtual Machine

### VM Loop Structure

```c
JSValue JS_Call(JSContext *ctx, int call_flags)
{
    JSValue *sp = ctx->sp;
    JSValue *fp = ctx->fp;
    uint8_t *pc;
    JSFunctionBytecode *b;
    // ...

    // Computed goto dispatch (GCC/Clang)
#ifdef __GNUC__
    #define CASE(op) L_ ## op
    #define BREAK goto *dispatch_table[*pc++]

    static const void *dispatch_table[] = {
        &&L_OP_invalid,
        &&L_OP_push_value,
        // ... all opcodes
    };

    BREAK;  // Jump to first opcode

    CASE(OP_push_value):
        // Implementation
        BREAK;
    // ...

#else
    // Switch-based dispatch
    for (;;) {
        switch (*pc++) {
        case OP_push_value:
            // Implementation
            break;
        // ...
        }
    }
#endif
}
```

**Computed Goto:**
- Faster than switch (no bounds check)
- Better branch prediction
- GCC/Clang extension

### Call Frames

**Frame Layout:**
```
High Address
[argument N-1]
...
[argument 0]
[function object]      ← FRAME_OFFSET_FUNC_OBJ
[this value]           ← FRAME_OFFSET_THIS
[saved PC]             ← FRAME_OFFSET_CUR_PC
[call flags]           ← FRAME_OFFSET_CALL_FLAGS
[previous fp]          ← FRAME_OFFSET_PREV_FP, fp points here
[local 0]
[local 1]
...
[temp values]
Low Address            ← sp
```

**Call Sequence:**

1. **Caller:**
   - Push arguments (in order)
   - Push function object
   - Push `this` value
   - Jump to `OP_call` or `OP_call_method`

2. **VM (OP_call):**
   - Push saved PC
   - Push call flags (argc, constructor bit)
   - Push previous fp
   - Set fp = sp
   - Jump to function code

3. **Callee:**
   - Allocate locals
   - Execute bytecode
   - Push return value
   - Execute `OP_return`

4. **VM (OP_return):**
   - Pop return value
   - Restore sp, fp, pc
   - Continue caller

### C Function Calls

**C Function Signature:**
```c
typedef JSValue JSCFunction(JSContext *ctx, JSValue *this_val,
                             int argc, JSValue *argv);
```

**Calling Convention:**
- `this_val`: Pointer to `this` on stack
- `argc`: Argument count (may have FRAME_CF_CTOR bit)
- `argv`: Pointer to arguments on stack

**From Bytecode:**
When calling C function:
1. Prepare arguments on stack (like JS function)
2. Identify C function (short func or object)
3. Look up in `c_function_table`
4. Add missing arguments (undefined)
5. Call C function pointer
6. Handle return value or exception

**C Function Types:**
- `JS_CFUNC_generic`: Basic function
- `JS_CFUNC_generic_magic`: With magic number
- `JS_CFUNC_constructor`: Constructor
- `JS_CFUNC_constructor_magic`: Constructor with magic
- `JS_CFUNC_generic_params`: With params object
- `JS_CFUNC_f_f`: Math function (double → double)

### Exception Handling

**Throwing:**
```c
JSValue JS_Throw(JSContext *ctx, JSValue obj)
{
    ctx->current_exception = obj;
    return JS_EXCEPTION;
}
```

**Catching:**
Bytecode sets up exception handler:

```
    catch label
        [protected code]
        goto end_label
    label:
        [exception in accumulator]
        [handler code]
    end_label:
```

**Stack Unwinding:**
When exception thrown:
1. VM checks for catch handler in current frame
2. If found, jump to handler with exception in accumulator
3. If not found, return to caller
4. Repeat until caught or top level

**Finally Blocks:**
Uses `OP_gosub` and `OP_ret`:
```
    gosub finally_label
    [normal code]
    goto end_label
finally_label:
    [cleanup code]
    ret
end_label:
```

### Tail Call Optimization

**Detecting Tail Calls:**
Parser recognizes tail position:
- Last expression in function
- No finally blocks active
- Return statement

**Execution:**
Instead of creating new frame:
1. Reuse current frame
2. Overwrite arguments
3. Reset locals
4. Jump to function start

**Limitations:**
Only within same bytecode function (not cross-function).

## Execution Flow Example

**Source:**
```javascript
function add(a, b) {
    return a + b;
}
add(1, 2);
```

**Bytecode (Simplified):**
```
// Function 'add':
  get_arg 0         // a
  get_arg 1         // b
  add
  return

// Global:
  push_const 0      // function 'add'
  fclosure          // create closure
  put_loc 0         // store in local

  push_i8 1         // arg 1
  push_i8 2         // arg 2
  get_loc 0         // load 'add'
  push_undef        // this
  call 2            // call with 2 args
```

**Execution Trace:**
```
1. Parse source → JSFunctionBytecode
2. Create global closure
3. Execute global code:
   - Create 'add' closure
   - Store in local 0
   - Push arguments
   - Load 'add' from local
   - Call with 2 arguments
4. Call 'add':
   - Create frame
   - Execute add bytecode
   - Return result
5. Continue global code
```

## Optimization Techniques

### Fast Paths

**Integer Arithmetic:**
```c
CASE(OP_add):
    op1 = sp[1];
    op2 = sp[0];
    if (likely(JS_VALUE_IS_BOTH_INT(op1, op2))) {
        // Fast path: both integers
        if (unlikely(__builtin_add_overflow(...)))
            goto add_slow;
        sp[1] = result;
    } else {
    add_slow:
        // Slow path: type conversion
    }
```

**Array Access:**
```c
CASE(OP_get_array_el):
    obj = sp[1];
    idx = sp[0];
    if (JS_IsPtr(obj) && JS_IsInt(idx)) {
        JSObject *p = JS_VALUE_TO_PTR(obj);
        if (p->class_id == JS_CLASS_ARRAY) {
            // Fast path: array[int]
            return array->arr[idx];
        }
    }
    // Slow path: general property access
```

**Property Access:**
Inline cache for property lookups (shape-based).

### Interrupt Handling

```c
#define POLL_INTERRUPT() do {                  \
    if (unlikely(--ctx->interrupt_counter <= 0)) { \
        if (ctx->interrupt_handler &&          \
            ctx->interrupt_handler(ctx, ctx->opaque)) \
            goto exception;                    \
        ctx->interrupt_counter = JS_INTERRUPT_COUNTER_INIT; \
    }                                          \
} while (0)
```

Called periodically to:
- Check for user interruption
- Yield control
- Enforce time limits

## Rust Port Considerations

### Bytecode Representation

```rust
enum Opcode {
    PushValue { value: JSValue },
    PushI8 { value: i8 },
    GetField { index: u16 },
    Add,
    // ... all opcodes
}

struct Bytecode {
    ops: Vec<u8>,
}

impl Bytecode {
    fn decode(&self, pc: usize) -> (Opcode, usize) {
        // Decode instruction at pc
        // Return (opcode, new_pc)
    }
}
```

### VM Loop

**Option 1: Match-based (portable)**
```rust
loop {
    let (op, new_pc) = bytecode.decode(pc);
    pc = new_pc;

    match op {
        Opcode::Add => {
            let b = stack.pop();
            let a = stack.pop();
            stack.push(a + b);
        }
        // ...
    }
}
```

**Option 2: Function pointers (faster?)**
```rust
type OpHandler = fn(&mut VM, operands: &[u8]);

static DISPATCH_TABLE: &[OpHandler] = &[
    op_push_value,
    op_add,
    // ...
];
```

### Parser as Iterator

```rust
struct Parser<'a> {
    source: &'a str,
    pos: usize,
    // ...
}

impl<'a> Parser<'a> {
    fn parse_statement(&mut self) -> Result<(), ParseError> {
        match self.current_token()? {
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            // ...
        }
    }
}
```

### Type Safety for Operands

```rust
struct ConstIndex(u16);
struct LocalIndex(u16);
struct LabelId(u32);

enum Operand {
    Const(ConstIndex),
    Local(LocalIndex),
    Label(LabelId),
    I8(i8),
    U16(u16),
}
```

### Error Handling

```rust
enum VMError {
    TypeError(String),
    ReferenceError(String),
    OutOfMemory,
    StackOverflow,
}

type VMResult<T> = Result<T, VMError>;
```

### Safety Invariants

Document invariants for unsafe code:
- Stack bounds checking
- Frame pointer validity
- GC safety during execution
- No raw pointer aliasing
