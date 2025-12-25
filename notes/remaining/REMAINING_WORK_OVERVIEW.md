# CrabQuick Remaining Work Overview

This document tracks the remaining implementation work for CrabQuick to reach feature parity with MicroQuickJS.

## Priority 1: Function Calls ✅ COMPLETE

See: [FUNCTION_CALLS_COMPLETE.md](./FUNCTION_CALLS_COMPLETE.md)

**Status**: All 5 phases implemented and tested

**What Works**:
- Function declarations compile to bytecode
- Function objects stored on heap
- Function calls with parameters
- Local variables inside functions
- Recursive functions (factorial, fibonacci)

---

## Priority 2: Type Coercion

### Current State
- Operators work only on matching types
- `"5" + 3` fails or gives wrong result
- `5 == "5"` doesn't apply Abstract Equality

### Files to Modify
- `crabquick/src/runtime/conversion.rs` - Implement ToNumber, ToString, ToBoolean
- `crabquick/src/runtime/operators.rs` - Apply conversions before operations
- `crabquick/src/runtime/compare.rs` - Implement Abstract Equality (==)

### Specific TODOs
```
runtime/conversion.rs:7:    // TODO: Implement ToNumber abstract operation
runtime/conversion.rs:13:   // TODO: Implement ToInt32 abstract operation
runtime/conversion.rs:19:   // TODO: Implement ToString abstract operation
runtime/conversion.rs:25:   // TODO: Implement ToBoolean abstract operation
runtime/operators.rs:7:     // TODO: Implement addition with type coercion
runtime/compare.rs:13:      // TODO: Implement abstract equality with type coercion
```

### Test Cases
```javascript
"5" + 3      // should be "53" (string concat)
5 + "3"      // should be "53"
5 - "3"      // should be 2 (numeric)
5 == "5"     // should be true
5 === "5"    // should be false
!!"hello"    // should be true
!!0          // should be false
```

---

## Priority 3: Built-in Methods

### Array.prototype Methods
```
runtime/init.rs:119: // TODO: Install Array.isArray
runtime/init.rs:120: // TODO: Install Array.prototype methods (push, pop, shift, etc.)
builtins/array.rs:216-242: // TODO: Implement callback execution (map, filter, etc.)
```

**Required for basic usage:**
- `push`, `pop`, `shift`, `unshift`
- `slice`, `splice`, `concat`
- `indexOf`, `includes`
- `length` property

### String.prototype Methods
```
runtime/init.rs:141: // TODO: Install String.prototype methods
```

**Required:**
- `charAt`, `charCodeAt`
- `slice`, `substring`, `substr`
- `indexOf`, `lastIndexOf`
- `toLowerCase`, `toUpperCase`
- `split`, `trim`
- `length` property

### Object Static Methods
```
runtime/init.rs:98: // TODO: Install Object static methods (keys, values, entries, etc.)
```

**Required:**
- `Object.keys`
- `Object.values`
- `Object.entries`
- `Object.assign`

### Function.prototype Methods
```
runtime/init.rs:203: // TODO: Install Function.prototype methods (call, apply, bind)
builtins/function.rs:12-28: // TODO: Implement call, apply, bind
```

---

## Priority 4: Object Internals

Many object types have stub implementations:

### JSString
```
object/string.rs: 11 TODOs
- Proper UTF-8 storage
- Length computation
- Hash caching
```

### JSArray
```
object/array.rs: 10 TODOs
- Dynamic backing storage
- Sparse array support
- Length property sync
```

### JSFunction
```
object/function.rs: 7 TODOs
- Bytecode function storage (addressed in function calls plan)
- Closure environment
```

---

## Priority 5: Global Functions

```
runtime/init.rs:331-338: // TODO: Create native function objects for:
  - parseInt
  - parseFloat
  - isNaN
  - isFinite
  - eval
  - encodeURI / decodeURI
  - encodeURIComponent / decodeURIComponent
```

Also:
```
runtime/globals.rs:131-165: URI encoding/decoding stubs
```

---

## Priority 6: Not Implemented Features

### JSON
```
builtins/json.rs:2: //! TODO: Implement JSON.parse and JSON.stringify
```

### RegExp
```
builtins/regexp.rs:2: //! TODO: Implement basic regexp engine
```

### TypedArrays
```
builtins/typed_array.rs:2: //! TODO: Implement Int8Array, Uint8Array, etc.
```

---

## Utility Functions

Low-priority but needed for correctness:

```
util/strtod.rs:5:  // TODO: Implement efficient number parsing
util/dtoa.rs:5:    // TODO: Implement efficient number formatting
util/utf8.rs:5-23: // TODO: UTF-8 utilities
```

---

## Summary by TODO Count

| Category | TODOs | Priority |
|----------|-------|----------|
| Function Calls | 0 | P1 ✅ COMPLETE |
| Type Coercion | ~10 | P2 |
| Built-in Methods | ~25 | P3 |
| Object Internals | ~30 | P4 |
| Global Functions | ~10 | P5 |
| JSON/RegExp/TypedArray | ~5 | P6 |

**Total: ~80 TODOs remaining**

---

## Recommended Order

1. **Function Calls** (current) - Enables real programs
2. **Type Coercion** - Required for JS semantics
3. **Array/String basics** - Most commonly used
4. **Global functions** - parseInt, isNaN, etc.
5. **Object methods** - Object.keys, etc.
6. **JSON** - Very commonly needed
7. **RegExp** - Complex, defer if possible
8. **TypedArrays** - Only if targeting WebAssembly interop
