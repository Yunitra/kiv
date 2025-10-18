# Design Principles

These principles guide every design decision in Kiv.

1. **Simple by default, powerful when needed**  
   The common case should be easy; advanced features are opt-in.

2. **Explicit over implicit**  
   No magic. If something happens, the code must show it.

3. **Zero-cost abstractions**  
   Abstractions must compile to the same code as hand-written low-level code.

4. **Consistency**  
   Similar problems → similar solutions. No special cases.

5. **Predictability**  
   Performance and behavior must be understandable from the source.

6. **Strong abstraction capability**  
   Users should be able to build powerful libraries without runtime overhead.

Violating these requires strong justification in an RFC.
