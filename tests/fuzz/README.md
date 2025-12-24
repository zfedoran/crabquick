# Fuzz Tests

Fuzzing targets for finding security vulnerabilities and edge cases.

## Targets

- Parser: Random JavaScript source code
- Bytecode decoder: Random bytecode sequences
- VM interpreter: Random instruction sequences
- GC allocator: Random allocation patterns

## Running

```bash
cargo install cargo-fuzz
cargo fuzz run parser
```

## Corpus

The corpus directory contains interesting test cases discovered by fuzzing.
