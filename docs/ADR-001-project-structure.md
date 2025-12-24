# ADR-001: Project Structure and Workspace Organization

**Status:** Accepted
**Date:** 2025-12-24
**Deciders:** MicroQuickJS Rust Port Team

## Context

MicroQuickJS is being ported from C to Rust as a native implementation. We need to decide on the project structure, how to organize the code into crates, and what dependencies (if any) to use.

The C version consists of a single ~18,000 line file (mquickjs.c) with supporting libraries. The Rust version needs to be more modular while maintaining the minimal footprint characteristic of MicroQuickJS.

## Decision Drivers

- **Minimal footprint**: Must maintain ability to run in ~10 kB RAM
- **Modularity**: Code should be well-organized and maintainable
- **Build separation**: Standard library compilation should be separate from core engine
- **Testing**: Need clear separation of test types
- **Developer experience**: Should be easy to build and develop

## Considered Options

### Option 1: Monolithic Single Crate

All code in one crate, similar to the C version.

**Pros:**
- Simple build process
- No inter-crate dependencies
- Easy to optimize as a whole

**Cons:**
- Poor code organization
- Mixing concerns (core, build tools, CLI)
- Harder to test in isolation
- Longer compile times

### Option 2: Workspace with Multiple Crates

Separate crates for core engine, build tools, and CLI.

**Pros:**
- Clear separation of concerns
- Can test each crate independently
- Parallel compilation
- Reusable core library
- Can have different dependency sets per crate

**Cons:**
- Slightly more complex project structure
- Need to manage workspace dependencies

### Option 3: Many Fine-Grained Crates

Each major subsystem as a separate crate.

**Pros:**
- Maximum modularity
- Very clear boundaries

**Cons:**
- Overly complex for this project
- Too many interdependencies
- Harder to optimize across boundaries
- Unnecessary overhead

## Decision

**We choose Option 2: Workspace with Multiple Crates**

The workspace will contain:

1. **mquickjs** - Core JavaScript engine library
   - No dependencies (zero-dependency policy)
   - Implements all core functionality
   - Public API for embedding

2. **mquickjs-build** - Build-time stdlib compiler
   - Depends on mquickjs
   - Compiles JavaScript stdlib to ROM bytecode
   - Emits Rust const data structures

3. **mqjs** - Command-line interface
   - Depends on mquickjs
   - REPL and script execution
   - Optional rustyline dependency for readline support

## Consequences

**Positive:**
- Clear separation between engine, tooling, and interface
- Core library can be used as a dependency by other projects
- Each crate can have appropriate dependencies (e.g., only CLI needs rustyline)
- Easier to test each component in isolation
- Build tools don't bloat the core library
- Parallel compilation speeds up development

**Negative:**
- Slightly more files to manage (3 Cargo.toml files)
- Need to maintain workspace configuration

**Neutral:**
- Test organization uses standard Rust conventions (tests/ directory)

## Implementation Notes

### Directory Structure

```
rustmicroquickjs/
├── Cargo.toml                 # Workspace root
├── mquickjs/                  # Core engine library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── memory/            # Memory management
│       ├── object/            # Object system
│       ├── bytecode/          # Bytecode format
│       ├── compiler/          # Parser and compiler
│       ├── vm/                # Virtual machine
│       ├── builtins/          # Standard library
│       ├── runtime/           # Runtime support
│       └── util/              # Utilities
├── mquickjs-build/            # Build tools
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── emit.rs
├── mqjs/                      # CLI
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── repl.rs
│       └── readline.rs
└── tests/                     # Test organization
    ├── unit/                  # Unit tests
    ├── integration/           # Integration tests
    ├── compliance/            # ECMAScript compliance
    └── fuzz/                  # Fuzzing targets
```

### Module Organization

The core library (mquickjs) is organized by functional area:
- **memory/**: Custom allocator and GC
- **object/**: Objects, properties, arrays, functions, strings
- **bytecode/**: Opcode definitions, encoding, constant pools
- **compiler/**: Lexer, parser, code generator, debug info
- **vm/**: Interpreter, stack, calls, exceptions
- **builtins/**: JavaScript standard library functions
- **runtime/**: Type conversions, operators, comparisons
- **util/**: UTF-8, number formatting, bit manipulation

### Zero-Dependency Policy

The core mquickjs crate has zero runtime dependencies. All functionality is implemented natively to:
- Maintain minimal footprint
- Reduce attack surface
- Ensure predictable behavior
- Enable no_std support
- Simplify auditing

Development dependencies (proptest, criterion) are allowed for testing and benchmarking.

### Build Configuration

The workspace uses:
- Rust edition 2021
- Shared workspace dependencies for dev tools
- Aggressive optimization in release mode (LTO, single codegen unit)
- Clippy lints enforced workspace-wide
- Rustfmt configuration for consistent style

## References

- [Implementation Plan Section 1: Project Structure](/root/rustmicroquickjs/notes/implementation-plan.md)
- [Cargo Workspaces Documentation](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [MicroQuickJS C Implementation](https://github.com/bellard/mquickjs)
