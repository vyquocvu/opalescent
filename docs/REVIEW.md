# Opalescent — Repository Review

> **Status update:** The blocking build failure in §1 has since been fixed by
> completing enum/match support. CI now gates on `cargo build --all-targets`
> and clippy, and the P1 recommendations have been addressed. This document is
> retained as the review snapshot.

Scope: full tree as of the current working state. This review is **read-only**; no source
changes were made. The in-progress enum/match work is reported as-is and was not modified.

Summary: the design dossier is unusually strong for a project at this stage, the
conformance suite is thoughtful, and the compiler pipeline is cleanly modular. However,
**the tree does not build**: an unfinished enum/match refactor leaves the working tree in
an inconsistent, non-compiling state, and CI is red as a result.

---

## 1. Blocking issue: build is broken

`cargo build --all` and `cargo test --all` both fail: `could not compile opalescent (lib) due to 11 previous errors`.

| Count | Error | Meaning |
|-------|-------|---------|
| 4 | `E0609` no field `enums` on type `&mut Analyzer<'a>` | `analysis.rs` references a field that does not exist |
| 2 | `E0609` no field `enum_names` on type `&mut Analyzer<'a>` | same |
| 3 | `E0599` no method `check_type` found for `&mut Analyzer<'a>` | `analysis.rs` calls a method that does not exist |
| 1 | `E0004` non-exhaustive `Stmt::Match { .. }` not covered | `analysis.rs` match missing the new variant |
| 1 | `E0004` non-exhaustive `ExprKind::Variant { .. }` not covered | `codegen.rs` match missing the new variant |

Exact sites:

- `E0609` `self.enums` at `src/analysis.rs:45:49`, `:71:18`, `:376:22` (compiler hint: `self.program.enums`).
- `E0609` `self.enum_names` at `src/analysis.rs:73:18`.
- `E0599` `self.check_type(...)` at `src/analysis.rs:78:22`, `:80:18`, `:186:22`.
- `E0004` at `src/analysis.rs` (match over `&ast::Stmt`).
- `E0004` at `src/codegen.rs:156` (match over `&expr.kind`).

Root cause: the `Analyzer` struct declares only `program`, `function_signatures`, `scopes`,
and `current_return_type`. There are no `enums` / `enum_names` fields, and no `check_type`
method. Newly added code assumes both exist. Meanwhile `codegen.rs` was left untouched by
the refactor, so it cannot handle the new `ExprKind::Variant` / `Stmt::Match` node kinds that
`ast.rs` now defines.

Uncommitted working-tree state (`git diff --stat` shows 5 files, 353 insertions, 4 deletions):

```
M src/analysis.rs   (+161)
M src/ast.rs        (+45)
M src/lexer.rs      (+4)
M src/parser.rs     (+139)
M src/token.rs      (+8)
```

Severity: **Critical.** Nothing downstream (tests, CI, benchmarks, examples) can run until
this is resolved. Note that `codegen.rs` is *not* in the modified set yet contains a match
that now fails to compile — a sign the refactor is only half-applied.

---

## 2. Architecture assessment

The compiler is a conventional six-stage pipeline exposed through `compile()` in `src/lib.rs`:

```
Lexer::tokenize -> Parser::parse -> Analyzer::analyze -> CodeGenerator::generate
```

- **Clean modularization.** Each stage lives in its own file (`lexer.rs`, `parser.rs`,
  `analysis.rs`, `codegen.rs`) with shared types in `token.rs`, `ast.rs`, `span.rs`,
  `diagnostics.rs`. Boundaries are clear and the pipeline is easy to follow.
- **Span propagation is real.** `Span { start, end, line, col }` flows through tokens, AST
  nodes and diagnostics, and `Diagnostic::render` produces a caret-gutter rendering. This is
  the kind of thing projects usually bolt on later; having it from the start is a strength.
- **Honest caveat.** As `docs/07-ARCHITECTURE.md` itself notes, the MVP emits Rust and lets
  `rustc` finish the job. It is a *Rust-emitting front end*, not a sound independent
  compiler. Several "language guarantees" (no hidden clones, ownership soundness) are
  currently enforced by the Rust emitter/borrow checker downstream rather than by
  Opalescent's own frontend. This is fine for a proof-of-concept, but the docs should keep
  saying so plainly.

Overall: good bones. The pipeline shape matches the architecture document rather than
diverging from it, which is more than most early-stage compilers manage.

---

## 3. Design dossier assessment

The `docs/` set is the strongest part of the repository.

- **A real status vocabulary.** `DECIDED-PROVISIONAL / CANDIDATE / OPEN / DEFERRED / REJECTED`
  is used consistently, so a reader can tell an aspirational sketch from a settled decision.
- **ADR 0001** (`docs/adr/0001-ownership-and-parameter-conventions.md`) is the standout: it
  resolves D001–D004 together, states the call-site convention (read → auto immutable borrow,
  mut → auto mutable borrow with `let mut` verification, take → move), records the rejected
  alternatives (Rust-style call-site `&`/`&mut`, implicit copy-on-escape, pure mutable-value),
  and ties each rule to codegen mappings and conformance tests.
- **Critical-path gating is explicit.** `docs/10-ROADMAP.md` puts the memory-model decision on
  the critical path before grammar/SPEC work, and `docs/09-DECISIONS.md` warns against closing
  D001–D004 independently. That is the right instinct.
- **Gap: implementation realizes a smaller language than the dossier.** The docs describe
  `enum`/`match`, ownership, and more; the code currently implements Int/Bool/String/Unit,
  functions, control flow, param modes, and `puts` — and the enum/match work is the very thing
  that is mid-flight and broken. The dossier is ahead of the code. That is an acceptable and
  even healthy ordering, but it means the docs should be read as *intent*, not *behavior*.

---

## 4. Tests, CI and benchmarks

- **`tests/conformance.rs` is genuinely good.** It carries a table of compile-fail fixtures
  with expected message fragments *and* expected 1-based line numbers, a positive corpus with
  needle assertions, a determinism check (`compile(src) == compile(src)`), an
  examples-compile sweep, and a `parser_never_panics_on_garbage` fuzz-ish list. Requiring
  error *messages* and *line numbers* rather than just "does it fail" is exactly right for a
  compiler with a diagnosis-quality goal.
- **Two fixtures disable the line check.** `injection` and `unclosed_if` use expected line `0`,
  which the harness treats as "skip the line assertion". Fine, but worth a comment so the
  omission stays deliberate.
- **CI is currently red** (`.github/workflows/ci.yml` runs `cargo test` plus three `cargo run`
  smoke commands on `ubuntu-latest`). Because the build fails, every CI job fails.
- **Benchmarks are directional, not rigorous.** `benchmarks/run.sh` builds `--release` and
  compares compile time, run time, and peak RSS against `rustc` and `ruby` on a Collatz loop
  (300,000 iterations). Useful as a sanity signal; not yet a controlled measurement (no warmup,
  repetitions, or statistical treatment).

---

## 5. Findings by severity

### Critical
- **Working tree does not compile** (11 errors; section 1). CI is red; nothing can be
  exercised.

### Medium
- **Incomplete enum/match refactor** is the direct cause: `ast.rs`/`parser.rs`/`lexer.rs`/
  `token.rs` gained nodes, `analysis.rs` partially assumes an `enums`/`enum_names`/`check_type`
  design that was never added to the struct, and `codegen.rs` was not updated at all.
- **Soundness/ownership guarantees are currently delegated to Rust.** The frontend does not
  itself prove the invariants the docs advertise; they hold only because generated Rust is
  checked by `rustc`. Acceptable for an MVP, but should be tracked as a known gap against
  `docs/05-OWNERSHIP.md`'s decision gate.

### Low
- **`enum_names` interning via `Box::leak`** (`TypeKind::Enum(&'static str)`) leaks per name for
  process lifetime — harmless for a short-lived compiler invocation, but a trap if the compiler
  is ever embedded or run in a long-lived process.
- **`puts` appears in the reserved-keyword list** (`token.rs`), which conflates "built-in
  function name" with "reserved keyword". Likely to bite once user-defined function names are
  considered.
- **Benchmark iteration count (300,000)** is small enough that noise may dominate; treat
  results as directional only.

---

## 6. Strengths

- Clear, honest project framing in `README.md` and `docs/00-VISION.md` (including explicit
  non-goals and success measures).
- A disciplined status/decision vocabulary across the entire `docs/` set.
- ADR 0001 is exemplary: resolves a cluster of decisions, records rejected alternatives, and
  links rules to tests and codegen.
- Real span propagation and caret diagnostics from day one.
- A conformance suite that asserts on message text and line numbers, plus determinism.

---

## 7. Prioritized recommendations

No changes were made as part of this review. Recommendations, highest impact first:

1. **Restore a green build before anything else.** Finish or park the enum/match refactor so
   `cargo test` passes again; then CI is meaningful. (The review intentionally did not perform
   this fix, per instructions to leave the WIP as-is.)
2. **Make the refactor atomic.** Add the missing `Analyzer` fields/method (or remove the
   references) *and* update `codegen.rs` in the same change, so no intermediate commit leaves
   the tree non-compiling.
3. **Add a regression guard for "no dangling match arms":** e.g. a CI check that `cargo build
   --all-targets` must pass on every push, so a half-applied AST change cannot land.
4. **Document the frontend-vs-rustc soundness boundary** explicitly in `docs/07-ARCHITECTURE.md`
   and `docs/05-OWNERSHIP.md`: state which invariants Opalescent proves and which are currently
   reliant on `rustc`.
5. **Revisit `Box::leak` interning** if the compiler is ever to run in a long-lived process.
6. **Keep `puts` out of the keyword set**, or document it as an intentional built-in.
7. **Harden the benchmark harness** (warmup, repetitions, report variance) before publishing
   any performance claims against Rust/Ruby.

---

## 8. Coordination artifacts

- Build: `cargo build --all`, `cargo test --all` (both currently failing).
- Tests: `tests/conformance.rs`.
- Examples: `examples/hello.opal`, `examples/flow.opal`, `examples/modes.opal`.
- Benchmarks: `benchmarks/run.sh` (+ `collatz.opal` / `collatz.rb` / `collatz.rs`).
- CI: `.github/workflows/ci.yml`.
