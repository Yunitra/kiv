# Design Goals

Kiv aims to be a **simpler alternative to Rust**, without sacrificing safety or performance.

## What "Simple" Means

- **Cognitive simplicity**: Fewer concepts to learn (e.g., no explicit lifetimes)
- **Syntactic simplicity**: Less punctuation, more readable code
- **Predictable behavior**: Same code → same result, always
- **Explicit over implicit**: No hidden control flow or automatic conversions

## Target Use Cases

- Systems programming (OS, drivers, embedded)
- Performance-critical applications
- Developers who find Rust powerful but complex

> Kiv is not trying to be "Rust but easier to learn".  
> It’s trying to be **"Rust’s safety and speed, with fewer sharp edges."**
