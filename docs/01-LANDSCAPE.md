# 01 — Comparative research

These are design comparisons, not equivalence claims.

| Language | Principle worth studying | Caution for Opalescent |
|---|---|---|
| Ruby | Human-focused, low-ceremony expressive syntax and block idioms | Dynamic dispatch and runtime metaprogramming cannot be copied into the safe, AOT core without explicit costs |
| Rust | Explicit ownership, borrowing, affine moves and compile-time alias control | Surface complexity, lifetime diagnostics and API learning burden |
| Hylo | Mutable value semantics, scoped access, implicit access analysis | Value/exclusivity semantics need rigorous mutation and projection specification |
| Swift | borrowing/consuming modifiers, ownership SSA and diagnostics | Swift may introduce implicit copies for copyable types; Opalescent's proposed cost policy differs |
| Vale/Valen | Alternative memory-safety strategies and the ergonomics vs runtime-check trade-off | Generational checks are not zero-cost; Vale is archived and research is evolving |
| Mojo | Ownership and explicit argument conventions in expressive syntax | Do not borrow branding or performance claims as proof of soundness |

## Lessons
1. Ergonomics is a property of *semantics + diagnostics*, not syntax alone.
2. Borrowing can be implicit at a call site when parameter modes are explicit in interfaces.
3. Moves, storage, escape and copying require stable, testable rules.
4. A Ruby-like aesthetic should not imply Ruby-equivalent object identity, dynamic dispatch or metaprogramming.
5. Backend choice does not prove frontend soundness: lowering must preserve every language rule.

See REFERENCES.md for primary material. Proposals below require independent validation and counterexamples.
