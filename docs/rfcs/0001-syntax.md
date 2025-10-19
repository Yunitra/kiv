# RFC 0001: Core Syntax and Language Style

- Feature name: Core Syntax Design
- Start date: 2025-10-18
- RFC PR: N/A (included in repository initialization)
- Kiv version target: 0.1
- Status: Accepted

## Summary

Define Kiv’s foundational syntax to achieve **readability, consistency, and simplicity**, guided by the principle: _“Simple by default, powerful when needed.”_  
This RFC establishes the look and feel of Kiv code, covering blocks, declarations, expressions, and naming.

## Motivation

A language’s syntax shapes how users think and write. Kiv aims to:
- Feel familiar yet distinct
- Minimize punctuation noise
- Be **expression-oriented**
- Enforce **explicitness** (e.g., explicit mutability, clear type annotations)

This foundation ensures future features integrate cohesively.

## Guide-level explanation

### Blocks and Scoping
- **Use curly braces `{}`** for all blocks (`fun`, `if`, `match`, etc.)
- **No significant indentation** (unlike Python)

```kiv
if x > 0 {
    println("positive")
} else {
    println("non-positive")
}
```

### Variable Declaration
- Use `let` for immutable bindings
- Use `let mut` for mutable bindings
- Type annotation uses `:` (same as function returns)

```kiv
let x: Int = 42            // immutable, explicit type
let mut y: Float = 3.14    // mutable
let name = "Kiv"           // inferred as Text
```

### Functions
- Defined with `fun`
- Parameters require type annotations
- Return type specified after `:`
- **Expression-oriented**: last expression is returned

```kiv
fun add(a: Int, b: Int): Int {
    a + b
}

fun greet(name: Text): Text {
    "Hello, " + name
}
```

### Control Flow as Expressions
- `if` and `match` are expressions
- All branches must return the same type

```kiv
let message = if score >= 90 {
    "Excellent"
} else {
    "Try again"
};
```

### Statement Termination
- **Semicolons are optional** and **not recommended**
- The language is designed to be clean and readable without semicolons
- Only add semicolons when absolutely necessary for disambiguation

```kiv
// Preferred style (no semicolons)
fun add(a: Int, b: Int): Int {
    let result = a + b
    return result
}

// Discouraged style (with semicolons)
fun add(a: Int, b: Int): Int {
    let result = a + b;
    return result;
}
```

### Naming and Built-in Types
- Built-in types use **PascalCase**: `Int`, `Float`, `Text`, `Bool`
- Variables/functions: `snake_case`
- Types (user-defined): `PascalCase`

### Comments
- `//` for line comments
- `///` for documentation comments (above items)
- `//!` for module-level docs (reserved for future)

```kiv
/// Adds two integers
fun add(a: Int, b: Int): Int {
    a + b
}
```

## Reference-level explanation

- Parser: Pratt parser for expression precedence
- Type inference: local only (no global inference)
- Mutability: tracked in type checker; mutable variables cannot be CoW-shared without `move`

## Drawbacks

- `fun` may feel unfamiliar to Rust/Go users (but distinct from `fn`)
- `:` for return types differs from C-family (but consistent with type annotations)

## Alternatives Considered

- `fn`: Rejected — too close to Rust; `fun` is clearer and shorter
- `->` for return: Rejected — inconsistent with `x: Int` style
- Implicit mutability: Rejected — violates “explicit over implicit”

## Unresolved questions

- How to format long function signatures? (Proposal: follow Go style)
