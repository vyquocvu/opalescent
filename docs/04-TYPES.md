# 04 — Static type and effect model

**DECIDED-PROVISIONAL:** static typing, local type inference, explicit public-facing signatures at least initially; no implicit dynamic fallback.

## Candidate type lattice
- Unit, Bool, fixed-width signed/unsigned integers, floating-point (exact widths OPEN).
- Owned String and string view; explicit allocation/lifetime story required.
- Product structs, tagged enums, generic parameters, Option<T> and Result<T,E>.
- References/access capabilities are part of typing, not a syntax-only desugaring.
- Traits/protocols for ad-hoc polymorphism; coherence/orphan rules OPEN.
- No implicit truthiness: conditions must be Bool (proposed).
- No implicit lossy numeric conversions (proposed).

## Inference boundary
Infer local bindings and generic instantiations when unique; require annotations for ambiguous public APIs and recursive definitions. Never infer an implicit clone, global heap promotion, or dynamic dispatch as a resolution strategy.

## Semantics to specify
- Type identity, variance, subtyping (if any), coercions, literal defaults, overflow, equality, object identity.
- Method lookup, trait dispatch, generic monomorphization vs alternatives.
- Type of closure capture, escaping borrowed values, async suspension.
- Exhaustive pattern matching and definite initialization.
- Result propagation and panic/exception distinction.
- Effects: IO, mutation, unsafe and async — OPEN whether represented in signatures or managed via scoped capabilities.

## Rejection corpus starters
Use-before-initialization, use-after-move, mismatched return, non-Bool condition, ambiguous overload, coercion narrowing, non-exhaustive match and escape of local borrow.
