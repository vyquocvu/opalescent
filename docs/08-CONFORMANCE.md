# 08 — Research validation and conformance

## Test categories
- Lexical/grammar golden cases incl. Unicode and malformed inputs.
- Static type positive and compile-fail cases.
- Ownership matrix: moves, borrowing, exclusivity, returns, partial moves, branches, closures, async.
- Runtime observable behavior: evaluation order, drops, overflow, panic, IO and errors.
- Differential programs against reference semantics / generated Rust where models align.
- Parser property tests/fuzzing; no crashes or source-code injection.
- Multi-target build tests and backend version matrix.

## Benchmark protocol
Compare semantically equivalent, independently reviewed programs, optimized release settings, controlled target CPU and allocation strategy. Report compiler version, source, workload, input sizes, warmup, confidence intervals, time, throughput, peak memory, allocation count, binary size and compile time. Compare to Rust and Ruby separately; distinguish throughput from authoring productivity.

## Human factors
Blind readability tasks; measure task completion time, errors, ability to explain ownership and diagnostic usefulness. Avoid optimizing for line count alone.

## Exit criteria before v0.1 language spec
Normative ownership/type rules, grammar, 50+ boundary corpus, diagnostic span fidelity, documented unsupported features, reproducible benchmark scripts and accepted ADRs for core open questions.
