# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- MVP compiler implementation with LLVM IR generation
- Support for basic types: `Int`, `Float`, `Bool`, `Text`, `Unit`
- Function declarations with parameters and return types
- Variable declarations with type annotations
- Expression statements and binary operations
- `print()` builtin function with multi-argument support
- `panic()` builtin function
- Copy-on-Write (CoW) Text runtime implementation
- Optional semicolons in statements
- Complete compilation pipeline: source → LLVM IR → object → executable

### Technical
- Modular compiler architecture with separate crates
- Span-based error reporting with source code snippets
- Type checking with proper TypeId mapping
- SSA form LLVM IR generation for Copy types
- Automatic runtime library compilation and linking
- GitHub Actions CI with LLVM 18.1.8 development kit
