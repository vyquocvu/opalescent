# 00 — Vision and constraints

## Hypothesis
Can an expressive, Ruby-readable, compiled language deliver statically enforceable memory safety and predictable native performance while transferring *incidental* ownership complexity from the author to the compiler?

This is **not** Ruby compatibility, not a Rust syntax skin, and not a promise that two fundamentally different semantics can be combined for free.

## Target user and use cases
Developers building CLI tools, backend services and native libraries who value concise code, static correctness and predictable deployment. Secondary: developers moving from expressive scripting to systems programming. Embedded, GPU, kernel and macro-heavy metaprogramming are not MVP targets.

## Success measures (research targets, not claims)
- All accepted safe-language conformance cases preserve defined memory and aliasing invariants; deliberately invalid cases fail with source-level errors.
- No compiler-inserted heap allocation, clone, reference counting or lock merely to repair ownership errors.
- Sample applications require materially less syntactic ceremony than equivalent safe Rust, evaluated with blinded readability tasks and measured source tokens; do not optimize only for line count.
- On specified representative workloads, compare optimized native binaries to equivalent Rust and report throughput, peak RSS, allocations, binary size, compile time. No universal 'as fast as Rust' claim without measurement.
- Diagnosis quality: user-visible span, why, and actionable remedy for ownership/type failures.

## Non-goals
- Running Ruby gems unchanged; monkey patching/open classes; unrestricted runtime method_missing or eval in safe core.
- Dynamic typing as implicit fallback for failures of type inference.
- Guaranteeing every valid Rust program can be written in Opalescent.
- Creating an independent backend before language semantics stabilize.
