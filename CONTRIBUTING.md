# Contributing to MicroQuickJS Rust

Thank you for your interest in contributing to MicroQuickJS Rust!

## Current Status

This project is currently in **Phase 0: Foundation** and is being developed following a detailed [Implementation Plan](notes/implementation-plan.md). The project is not yet accepting external contributions while the core architecture is being established.

## Future Contributions

Once the core implementation is complete (expected: late 2026), we will welcome contributions in the following areas:

- Bug fixes
- Performance improvements
- Additional ECMAScript features
- Documentation improvements
- Test coverage
- Platform-specific optimizations

## Development Setup

### Prerequisites

- Rust 1.70 or later
- cargo
- rustfmt
- clippy

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/rustmicroquickjs
cd rustmicroquickjs

# Build the project
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt
```

### Project Structure

See [ADR-001: Project Structure](docs/ADR-001-project-structure.md) for details on the workspace organization.

## Coding Standards

### Style

- Follow Rust standard style (enforced by rustfmt)
- Run `cargo fmt` before committing
- Configuration in `rustfmt.toml`

### Linting

- All clippy warnings must be addressed
- Run `cargo clippy` before committing
- Workspace lint configuration in root `Cargo.toml`

### Documentation

- All public APIs must be documented
- Use `//!` for module-level documentation
- Use `///` for item-level documentation
- Include examples where appropriate
- Mark incomplete examples with `rust,ignore`

### Safety

- Minimize unsafe code
- All unsafe blocks must have SAFETY comments explaining:
  1. Why unsafe is needed
  2. What invariants are maintained
  3. What could go wrong if misused
- Run MIRI tests on unsafe code: `cargo miri test`

### Testing

- Unit tests in `#[cfg(test)]` modules
- Integration tests in `tests/integration/`
- Aim for 85%+ test coverage
- Property-based tests using proptest where applicable
- Benchmarks using criterion

### Commit Messages

- Use clear, descriptive commit messages
- Reference issue numbers where applicable
- Follow conventional commits format:
  - `feat:` for new features
  - `fix:` for bug fixes
  - `docs:` for documentation
  - `test:` for tests
  - `refactor:` for refactoring
  - `perf:` for performance improvements

## Architecture Decision Records

Significant design decisions are documented as ADRs in the `docs/` directory. Use the template in `docs/ADR-template.md`.

## Questions?

For now, please refer to the [Implementation Plan](notes/implementation-plan.md) for roadmap and architecture details.

---

**Note:** This file will be updated once the project is ready for external contributions.
