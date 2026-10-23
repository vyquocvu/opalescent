# ADR 0002 — Native backend strategy

Status: accepted (exploration outcome for Milestone 5)

## Context
The compiler emits safe Rust and shells out to `rustc -C opt-level=3`. Benchmarks (`benchmarks/run.sh`, collatz) show runtime and peak RSS on par with hand-written Rust and about 4.7x faster than Ruby. Compile time is higher than plain `rustc` (~0.96s vs ~0.26s) because it includes our own front end plus invoking `rustc`.

## Options
1. **Keep Rust emission** — best runtime, inherits `rustc` optimizations and safety checks. Requires `rustc` at build time.
2. **Cranelift** — no `rustc` dependency, fast compile, weaker optimization. Needs a typed MIR, which does not exist yet.
3. **LLVM (inkwell)** — strongest optimization, heavy dependency and build cost.

## Decision
Keep the Rust-emission backend as the reference backend. Do not add Cranelift/LLVM until the language grows features that Rust emission cannot express (e.g. custom memory layouts) or a `rustc`-free toolchain becomes a goal. Any future backend must first lower from a typed MIR and pass the conformance suite plus `benchmarks/run.sh` with no runtime regression.

## Consequences
Milestone 5 is closed as an exploration with this recorded decision, not an implementation.
