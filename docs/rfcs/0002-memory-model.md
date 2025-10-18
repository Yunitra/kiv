# RFC 0002: Memory Management Model — Copy and Copy-on-Write

- Feature name: Copy and Copy-on-Write Memory Model
- Start date: 2025-10-18
- RFC PR: N/A (included in repository initialization)
- Kiv version target: 0.1
- Status: Accepted

## Summary

Kiv adopts a **two-tier memory model** based on **Copy** and **Copy-on-Write (CoW)** semantics. This model provides:
- **Memory safety** without garbage collection
- **Near-zero runtime overhead** for common patterns
- **Extreme simplicity** for users: no explicit ownership, borrowing, or lifetimes
- **Clear path to safe concurrency** in the future

All types are statically classified as either `Copy` or `CoW`. Assignment, parameter passing, and returns follow uniform, predictable rules.

## Motivation

Rust’s ownership system guarantees memory safety and zero-cost abstractions, but at the cost of significant cognitive load. Many developers desire:
- The **safety of Rust**
- The **simplicity of GC languages** (e.g., Python, JavaScript)
- The **performance of manual memory management**

Kiv’s model bridges this gap by:
- Making **memory management invisible in the common case**
- Ensuring **no use-after-free, no data races**
- Allowing **high performance through compiler optimizations**

This aligns with our core principle: _“Simple by default, powerful when needed.”_

## Guide-level explanation

### Two Kinds of Types

1. **`Copy` types**: Small, stack-allocated, trivially duplicable.
   - Examples: `i32`, `f64`, `(i32, bool)`, small arrays
   - Assignment copies the value:
     ```kiv
     let a: i32 = 42
     let b = a   // b is a new copy; a and b are independent
     ```

2. **`CoW` types**: Large, heap-allocated, shared until mutation.
   - Examples: `Text`, `List<T>`, `Map<K, V>`
   - Assignment shares the data; mutation triggers a copy:
     ```kiv
     let a = "hello".to_text()  // heap-allocated, RC=1
     let b = a                  // RC=2, no copy
     b.push('!')                // RC=2 > 1 → clone, then modify → b now owns new copy (RC=1)
     // a is unchanged; b is independent
     ```

### Key User Rules

- **No explicit memory management**: No `free`, no `clone()` unless desired.
- **Mutation is always safe**: You never see shared mutable state.
- **Performance is predictable**: Only the first write after sharing incurs a copy.

### Escape Hatch: `move`

To avoid CoW overhead when transferring ownership:
```kiv
let a = large_text()
let b = move a   // a is invalidated; b takes ownership — no RC change, no copy
```

## Reference-level explanation

### Type Classification

- **`Copy`**: Types with size ≤ 16 bytes and no heap pointers (configurable threshold).
- **`CoW`**: All other types (heap-allocated with reference-counted header).

### Runtime Representation

A `CoW` object has this layout:
```
+------------------+
| ref_count: usize |  // non-atomic by default
| data...          |
+------------------+
```

- **Reference counting is non-atomic** → zero synchronization cost in single-threaded code.
- On `move`, the reference count is **not incremented**; ownership is transferred.

### Compiler Optimizations

1. **Uniqueness analysis**: If the compiler proves an object has only one reference, it skips the CoW check and modifies in-place.
2. **Small String/Vector Optimization (SSO)**: Objects ≤ N bytes are stored inline (no heap, no RC).
3. **Elision of `move`**: Return of local CoW values is automatically moved.

### Safety Guarantees

- **No use-after-free**: RC ensures objects live as long as referenced.
- **No data races**: Mutation always produces a unique copy; no shared mutable state.
- **Deterministic destruction**: Objects are freed immediately when RC drops to 0.

## Performance Model

| Scenario                     | Cost vs Rust |
|------------------------------|--------------|
| Create small `Copy` value    | Identical (stack) |
| Create `CoW` value (SSO)     | Identical (stack) |
| Create large `CoW` value     | Slightly higher (RC init) |
| Read-only sharing            | Slightly higher (RC inc/dec) |
| **First write after share**  | **+1 clone** (one-time cost) |
| Subsequent writes            | Identical (unique ownership) |
| `move` transfer              | Identical (pointer transfer) |

> The model achieves **near-zero-cost abstraction** for typical usage, with a small, predictable cost only in shared-read scenarios — a deliberate trade-off for simplicity.

## Concurrency Considerations (Future-Proofing)

Although Kiv 0.1 is single-threaded, the model is designed for safe concurrency:

- **Message passing**: Sending a `CoW` value across threads uses `move` semantics (ownership transfer).
- **Shared immutable data**: Will be supported via `Arc<T>` (atomic RC), requiring `T: Sync`.
- **Shared mutable data**: Will require explicit synchronization (`Mutex<T>`, etc.).
- **No implicit sharing**: The compiler will prevent accidental cross-thread CoW sharing.
- **Object headers reserve space** to switch between non-atomic and atomic RC when needed.

## Drawbacks

- Slightly higher overhead than Rust in read-heavy, shared scenarios.
- Users cannot fine-tune memory layout (e.g., no custom allocators in 0.1).
- Not suitable for hard real-time systems requiring fully deterministic allocation (though SSO mitigates this).

## Alternatives Considered

1. **Rust-style ownership**: Rejected — violates “simple by default”.
2. **Tracing garbage collection**: Rejected — violates zero-cost and predictability.
3. **Hybrid (ownership + CoW)**: Rejected — adds complexity; we prefer uniformity.

## Unresolved questions

- What is the exact size threshold for `Copy` vs `CoW`? (Proposal: 16 bytes)
- Should SSO be enabled for all CoW types, or only `Text`/`List`?
- How to expose `move` in error messages? (e.g., “consider using `move` to avoid copy”)
