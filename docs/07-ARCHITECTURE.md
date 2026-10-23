# 07 — Compiler architecture proposal

```text
.opal source
  -> token stream + byte spans
  -> concrete syntax tree / error recovery
  -> AST + name resolution
  -> typed HIR + constraint solving
  -> ownership/access analysis + control-flow graph
  -> typed lowering IR + verifier
  -> Rust codegen (bootstrap)
  -> rustc -> native artifact
```

## Separation of concerns
Frontend defines Opalescent semantics; Rust output is a verified implementation strategy, not the definition of Opalescent. Every transformation must preserve evaluation order, move/borrow rules, destructor behavior, error behavior and source location attribution.

## Bootstrap milestones
- Split monolithic parser into lexer, AST, resolver, typechecker, ownership analysis, emitter, diagnostics and driver.
- Attach source spans to all nodes and maintain codegen source maps.
- Rust diagnostics are backend implementation failures until mapped into meaningful Opalescent errors.
- Snapshots/golden tests for parser, HIR and generated code; compile-fail fixture suite with expected spans.
- Verifier: no unresolved names/types/capabilities in typed IR.

## Toolchain contract
`opal check`, `opal build`, `opal run`, `opal fmt` (formatter deferred), package manifest/lockfile (deferred). Reproducible builds and backend-version compatibility need explicit policy.

## Caveat about present MVP
The current MVP is a small proof-of-concept Rust emitter, not a sound independent frontend; its limited parser, source mapping, semantic analysis and tests should not be mistaken for stable implementation of these proposals.

## Frontend vs rustc soundness boundary (current MVP)
The Opalescent frontend itself enforces: duplicate/undefined names, declared and expression types, enum type names and `match` coverage, parameter-mode compatibility at call sites, call-argument exclusivity for `mut`/`take`, and a conservative use-after-move check for consumed strings.

It does **not** yet prove the full ownership model. Borrow lifetimes, aliasing across nested expressions, partial moves, drop order and most of the access matrix are enforced by the generated Rust through `rustc`. A program accepted by `opal check` is therefore only as sound as the emitted Rust plus the frontend checks listed above. Closing this gap requires a typed HIR/MIR and an independent access analysis, as required by the decision gate in [05-OWNERSHIP.md](05-OWNERSHIP.md).
