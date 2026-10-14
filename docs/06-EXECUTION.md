# 06 — Execution, safety and interop

## Candidate execution model
AOT native compilation; platform and ABI targets OPEN. No required tracing GC for the core. Destruction order, stack/heap behavior, bounds checks, panic strategy and integer overflow must be normative, not inherited accidentally from the chosen Rust profile.

## Safe boundary
- Safe core forbids raw-pointer dereference and arbitrary runtime code injection.
- Unsafe facilities (if introduced) require an explicit syntactic boundary and an auditable contract; safe APIs wrapping unsafe code carry proof/review obligations.
- FFI is deferred until ABI, string/array ownership, callbacks, unwind and thread-safety contracts are specified.
- No arbitrary Rust source escape hatch inside safe Opalescent.
- Binary interoperability does not mean arbitrary Rust crate source compatibility.

## Costs
Define a cost model for allocation, deep/shallow copy, reference counting, dynamic dispatch, async state machines and bounds checking. An optimizing compiler may remove costs but cannot make a source-level effect observable contrary to spec.

## Concurrency
No automatic claim of race freedom for external libraries or unsafe boundaries. Actor/channel/thread APIs require Send/Sync-equivalent capability rules; async scheduling and cancellation semantics remain OPEN.

## Portability
Define deterministic behavior vs implementation-defined details; separate safety from functional correctness and resource exhaustion.
