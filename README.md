# MicroQuickJS - Rust Port

A native Rust implementation of MicroQuickJS, a minimal JavaScript engine designed for extremely constrained environments.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

## Overview

MicroQuickJS Rust is a from-scratch port of [MicroQuickJS](https://github.com/bellard/mquickjs) from C to idiomatic Rust. It maintains the minimal resource footprint (10-12 kB RAM minimum) while leveraging Rust's safety guarantees.

**Status:** Phase 0 - Foundation (In Development)

### Key Features

- **Minimal footprint**: Runs in as little as 10-12 kB RAM
- **ES5 compatible**: Supports ECMAScript 5.1 strict mode
- **Custom GC**: Mark-and-compact garbage collector optimized for small heaps
- **Zero dependencies**: Core library has no external dependencies
- **Memory safe**: Written in Rust with isolated unsafe code
- **Embedded-friendly**: Designed for resource-constrained environments

## Architecture

The engine consists of several major subsystems:

- **Memory Management**: Custom bump allocator with mark-and-compact GC
- **Value System**: Tagged union representation for efficient value encoding
- **Object System**: Property hash tables and prototype chain support
- **Bytecode**: Compact instruction format with ~260 opcodes
- **Compiler**: One-pass JavaScript parser and bytecode generator
- **Virtual Machine**: Stack-based bytecode interpreter
- **Built-ins**: JavaScript standard library implementation
- **Runtime**: Type conversions and operator implementations

## Project Structure

This is a Cargo workspace with three crates:

- **mquickjs** - Core JavaScript engine library (zero dependencies)
- **mquickjs-build** - Build-time compiler for ROM-resident standard library
- **mqjs** - Command-line REPL and script executor

## Building

```bash
# Build all crates
cargo build

# Build optimized release
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt
```

## Usage

### As a Library

```rust
use mquickjs::Context;

fn main() {
    // Create a context with 8 KB of memory
    let mut ctx = Context::new(8192);

    // Evaluate JavaScript code
    let result = ctx.eval("2 + 2", "script.js", 0);

    // Use the result
    if let Some(n) = result.to_int() {
        println!("Result: {}", n);
    }
}
```

### Command-Line Interface

```bash
# Run a JavaScript file
mqjs script.js

# Start the REPL
mqjs --repl

# Execute with memory limit
mqjs --memory 4096 script.js
```

## Development Status

### Phase 0: Foundation ✅ (Current)

- [x] Project structure and workspace setup
- [x] Module organization
- [x] Public API skeleton
- [x] Build configuration (rustfmt, clippy)
- [x] Test directory structure
- [x] Documentation infrastructure (ADRs)

### Phase 1: Memory Management (Next)

- [ ] Memory block headers with bit packing
- [ ] Bump allocator
- [ ] Mark-and-compact garbage collector
- [ ] GC root handle system
- [ ] Comprehensive testing

### Future Phases

See [Implementation Plan](notes/implementation-plan.md) for detailed roadmap.

## Design Decisions

Key architectural decisions are documented as Architecture Decision Records (ADRs):

- [ADR-001: Project Structure](docs/ADR-001-project-structure.md)

## Testing

The project uses multiple testing strategies:

- **Unit tests**: In-source `#[cfg(test)]` modules and tests/unit/
- **Integration tests**: tests/integration/
- **Compliance tests**: ECMAScript test suite in tests/compliance/
- **Fuzz tests**: Fuzzing targets in tests/fuzz/
- **Property-based testing**: Using proptest
- **Benchmarks**: Using criterion

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Run fuzzer (requires cargo-fuzz)
cargo fuzz run parser
```

## Performance Goals

- Execution speed: Within 1.2x of C version
- Memory overhead: < 20% vs C version
- GC pause time: < 2ms for 64KB heap
- Minimum RAM: 10-12 kB (allowing 20% overhead)

## Safety Strategy

The codebase minimizes unsafe code:

- Public API is 100% safe
- Unsafe code isolated to memory management and GC
- All unsafe blocks extensively documented
- MIRI testing for undefined behavior
- Fuzz testing for edge cases

## Contributing

This is currently a solo development project following a detailed implementation plan. Contributions will be welcome once the core implementation is complete.

## Documentation

- [Implementation Plan](notes/implementation-plan.md) - Detailed 42-week roadmap
- [Architecture Decision Records](docs/) - Design decisions
- [API Documentation](https://docs.rs/mquickjs) - Generated from code

## License

MIT License - See [LICENSE](LICENSE) for details.

## Acknowledgments

- Based on [MicroQuickJS](https://github.com/bellard/mquickjs) by Fabrice Bellard
- Inspired by [QuickJS](https://bellard.org/quickjs/)
- Rust ecosystem tools and libraries

## Resources

- [ECMAScript 5.1 Specification](https://262.ecma-international.org/5.1/)
- [Original MicroQuickJS](https://github.com/bellard/mquickjs)
- [QuickJS Documentation](https://bellard.org/quickjs/quickjs.html)

---

**Note:** This is a work in progress. The engine is not yet functional. See the implementation plan for current status and timeline.
