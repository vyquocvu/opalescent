# 05 — Ownership and memory model (primary research axis)

**Status: OPEN for concrete semantics.** Do not ship marketing claims of Rust-equivalent safety before proof obligations and compile-fail tests exist.

## Proposed invariants
- Each non-Copy owned value has one logical owner at a point in the abstract execution.
- A move invalidates the moved-from place until a legal reinitialization; partial moves/projections need explicit rules.
- Multiple shared accesses OR a single exclusive mutable access; never conflicting live access to the same storage.
- Borrow cannot outlive its owner; destruction runs exactly once for initialized owned resources, including early exits and panic paths where applicable.
- Race-free safe concurrency only for safe types/capabilities that satisfy explicit cross-thread rules.
- No automatic Clone/Rc/Arc/Box insertion solely to satisfy ownership.

## Parameter modes — design proposal
`read T` / `mut T` / `take T` (names OPEN). A call site may omit borrow notation when callee mode is statically known. A consuming call must be recognizable without global flow analysis; whether `move x` is mandatory at site remains OPEN.

```ruby
def inspect(value: read String)
  puts value
end

def consume(value: take String)
  # owner transferred
end
```

This illustration is **not implemented**. A call cannot automatically borrow a consuming argument; doing so would change the callee contract.

## Evaluation matrix
1. Shared read, nested read, sequential mutable access, overlapping access.
2. Branch-sensitive move and reinitialization.
3. Returning a view of a local vs of a parameter.
4. Struct-field projections and alias analysis.
5. Captured closures; escaping captures.
6. Recursive types and self-reference.
7. Graphs, callbacks, observers and cyclic references.
8. Async suspension, threads, channels and FFI.
9. Destructors, panic/unwind, partial initialization.

## Alternatives under research
A. Rust-style affine ownership with elided call-site borrows.
B. Hylo-style mutable value semantics and scoped access.
C. Swift-like explicit borrowing/consuming with restricted implicit copy.
D. Region / reference-count / generational hybrid only if costs and guarantees explicitly change.

## Decision gate
Build a corpus of 50+ positive/negative programs; define formal small-step operational rules or an equivalent ownership judgment; demonstrate sound lowering to generated Rust with an auditable mapping. Record counterexamples to any inference heuristic.

## Current enforcement boundary (MVP)
The frontend currently checks parameter-mode compatibility at call sites, rejects immutable-to-`mut` calls, detects repeated `mut`/`take` arguments in one call, and tracks use-after-move for consumed strings. It does **not** independently prove borrow lifetimes, nested-expression aliasing, partial moves, drop order or the wider access matrix; those are delegated to `rustc` through the generated Rust. Until a typed HIR/MIR and independent access analysis exist, "sound" claims apply to the frontend checks above plus the emitted Rust, not to Opalescent semantics as a whole.
