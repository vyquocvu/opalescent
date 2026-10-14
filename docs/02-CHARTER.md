# 02 — Language charter

**Working slogan:** Human-first syntax; machine-checkable semantics.

## Order of priorities (DECIDED-PROVISIONAL)
1. Soundness of the defined safe subset.
2. Transparent observable costs.
3. Readability and local reasoning.
4. Predictable optimized native execution.
5. Expressive abstraction.
6. Compatibility and brevity.

A readability feature must not silently violate a higher-ranked item. Performance is measured rather than assumed.

## Rules
- Static type checking; no fallback to implicit dynamic `Any`.
- Immutable binding by default; mutation marked locally.
- No silent clone or heap promotion to fix ownership errors.
- Explicit ownership transfer at semantic boundaries; site-level notation is an open choice.
- Safe programs cannot access freed memory, double free or cause data races under the normative memory model.
- Compile-time diagnostics must map back to Opalescent spans.
- Language semantics, standard library and backend code generation are separately versioned concepts.
- Features must have acceptance/rejection examples before syntax is frozen.

## Design tension
Ruby's dynamic semantics and Rust's static guarantees are not both preserved. We take Ruby's expressive *presentation*, while constructing separately specified static semantics. The compiler may infer safe, cost-free access modes only under explicit, finite rules.
