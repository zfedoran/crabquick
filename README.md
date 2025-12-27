# CrabQuick

A native Rust implementation of a minimal JavaScript engine designed for extremely constrained environments.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

## Overview

CrabQuick is a from-scratch port of [MicroQuickJS](https://github.com/bellard/mquickjs) from C to idiomatic Rust. It maintains the minimal resource footprint (10-12 kB RAM minimum) while leveraging Rust's safety guarantees.

-----------------------------
**Status:** Active Development

Core engine functional with 300+ passing tests

Summary

The engine is ~80% complete:
  - Core architecture is solid and tested
  - Simple expressions and Math functions work

Missing for mquickjs parity:

| Priority | Feature                                                                                         | Complexity |
|----------|-------------------------------------------------------------------------------------------------|------------|
| 1        | String methods (charAt, indexOf, slice, split, substring, toLowerCase, trim, replace)           | Medium     |
| 2        | More Array methods (shift, unshift, concat, slice, splice, indexOf, sort, reverse, reduceRight) | Medium     |
| 3        | Object methods (Object.keys, Object.create, Object.getPrototypeOf, hasOwnProperty)              | Medium     |
| 4        | for...of / for...in                                                                             | Medium     |
| 5        | Function.call/apply/bind                                                                        | Medium     |
| 6        | Number methods (parseInt, parseFloat, toFixed)                                                  | Easy       |
| 7        | JSON.parse/stringify                                                                            | Hard       |
| 8        | RegExp                                                                                          | Hard       |
| 9        | Typed Arrays                                                                                    | Hard       |
| 10       | Error types (TypeError, ReferenceError, etc.)                                                   | Easy       |
| 11       | Date.now()                                                                                      | Easy       |
| 12       | Exponentiation                                                                                  | Easy       |

-----------------------------

### Key Features

- **Minimal footprint**: Runs in as little as 10-12 kB RAM
- **ES5 compatible**: Supports ECMAScript 5.1 strict mode
- **Custom GC**: Index-based mark-and-compact garbage collector
- **Zero dependencies**: Core library has no external dependencies
- **Memory safe**: Written in Rust with isolated unsafe code
- **Embedded-friendly**: Designed for resource-constrained environments

## Architecture

The engine consists of several major subsystems:

- **Memory Management**: Custom bump allocator with index-based GC
- **Value System**: NaN-boxed tagged value representation
- **Object System**: Property hash tables and prototype chain support
- **Bytecode**: Compact instruction format with 104 opcodes
- **Compiler**: One-pass JavaScript parser and bytecode generator
- **Virtual Machine**: Stack-based bytecode interpreter
- **Built-ins**: JavaScript standard library (Math, console, etc.)
- **Runtime**: Type conversions and operator implementations

## Project Structure

This is a Cargo workspace with three crates:

- **crabquick** - Core JavaScript engine library (zero dependencies)
- **crabquick-build** - Build-time compiler for ROM-resident standard library
- **crabquick-cli** - Command-line REPL and script executor

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
use crabquick::Engine;

fn main() {
    // Create an engine with 64 KB heap
    let mut engine = Engine::new(65536);

    // Execute JavaScript code
    match engine.eval("2 + 3") {
        Ok(result) => println!("Result: {:?}", result),
        Err(error) => eprintln!("Error: {:?}", error),
    }

    // Get string result
    let text = engine.eval_as_string("Math.PI").unwrap();
    println!("Pi: {}", text);

    // Get memory statistics
    let stats = engine.memory_stats();
    println!("Heap: {} / {}", stats.heap_used, stats.heap_size);
}
```

### Command-Line Interface

```bash
# Run a JavaScript file
crabquick examples/hello.js

# Execute code directly
crabquick -e "console.log('Hello, World!')"

# Run with memory statistics
crabquick -m examples/fibonacci.js

# Show help
crabquick --help
```

## Development Status

### Completed

- Memory management with index-based GC
- NaN-boxed value system
- Object system with property tables
- Bytecode system (104 opcodes)
- Stack-based virtual machine
- One-pass compiler (lexer, parser, codegen)
- Runtime initialization with built-ins
- Native function support (Math.abs, console.log, etc.)

### In Progress

- Extended built-in method coverage
- Performance optimization
- Error handling improvements

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Design Decisions

Key architectural decisions are documented in the [notes/](notes/) directory:

- [Architecture Overview](notes/design/architecture.md)
- [Memory Management](notes/design/memory-management.md)
- [Implementation Plan](notes/implementation-plan.md)

## Safety Strategy

The codebase minimizes unsafe code:

- Public API is 100% safe
- Unsafe code isolated to memory management and GC
- All unsafe blocks extensively documented
- Index-based references for GC safety

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Documentation

Implementation notes and phase summaries are in the [notes/](notes/) directory.

## License

MIT License - See [LICENSE](LICENSE) for details.

## Acknowledgments

- Based on [MicroQuickJS](https://github.com/bellard/mquickjs) by Fabrice Bellard
- Inspired by [QuickJS](https://bellard.org/quickjs/)

## Resources

- [ECMAScript 5.1 Specification](https://262.ecma-international.org/5.1/)
- [Original MicroQuickJS](https://github.com/bellard/mquickjs)
- [QuickJS Documentation](https://bellard.org/quickjs/quickjs.html)

## Examples

See the [examples/](examples/) directory for sample JavaScript programs.

```bash
cargo run --package crabquick-cli -- examples/hello.js
```
