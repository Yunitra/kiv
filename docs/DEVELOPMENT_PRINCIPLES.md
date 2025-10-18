# Development Practices for Kiv

This document outlines how we build Kiv — our engineering practices, testing strategy, and quality standards.

## Core Development Model

We combine multiple proven practices:

- **Test-Driven Development (TDD)** for compiler internals  
  Every semantic rule, type checker, and IR generator is driven by tests.
  
- **Behavior-Driven Development (BDD)** for user-facing features  
  Language behavior is specified in human-readable scenarios (e.g., “assigning a CoW value should not clone”).

- **Extreme Programming (XP) Engineering Practices**  
  - Continuous Integration (CI)
  - Refactoring
  - Simple design (YAGNI)
  - Pair programming / thorough code review

This ensures Kiv is **correct, maintainable, and user-focused**.

## Testing Strategy

### 1. Unit Tests (`#[test]`)
- For low-level components (lexer, parser, IR builder)
- Use Rust’s built-in test harness

### 2. Integration Tests (`tests/`)
- End-to-end compilation tests
- Verify generated LLVM IR is valid

### 3. Compile-Fail Tests (`trybuild`)
- Test expected compilation errors
- Ensure error messages are clear and actionable
- Example: `tests/ui/move_after_use.kiv` + `.stderr`

### 4. Property-Based Tests (`proptest`)
- Validate invariants (e.g., “all expressions produce valid IR”)

## Code Style

- Follow Rust’s official style (`rustfmt`)
- Use `clippy` for lints
- PascalCase for types, snake_case for functions/variables (in Kiv code and compiler)

## CI Requirements

Every PR must pass:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `trybuild` UI tests

## Documentation

- RFCs are the source of truth for design
- User documentation (tutorials, reference) will be built with `mdBook`
- **Document as you go** — no “I’ll write docs later”

> This is not bureaucracy — it’s how we keep Kiv **simple, predictable, and reliable**.
