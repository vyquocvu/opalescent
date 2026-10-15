# Opalescent language MVP

Opalescent aims for Ruby-shaped readability with Rust-like compile-time safety and native performance. The current implementation is an **experimental subset**, not full Ruby or a replacement for Rust.

## Example

```ruby
def factorial(n: Int) -> Int
  let mut result: Int = 1
  let mut i: Int = 2
  while i <= n
    result = result * i
    i = i + 1
  end
  return result
end

def main()
  puts factorial(5)
end
```

## Commands

Prerequisites: recent stable Rust and `rustc` on PATH.

```sh
cargo build
cargo run -- run examples/hello.opal
cargo run -- check examples/flow.opal
cargo run -- emit-rust examples/hello.opal
cargo run -- build examples/hello.opal -o hello
./hello
```

## Implemented subset

- `def name(arg: Type) -> Type ... end` top-level functions; `def main()` entry point.
- Parameter modes: `read T` (borrowed immutable), `mut T` (borrowed mutable), `take T` (consumed ownership).
- `Int` (Rust `i64`), `Bool` (Rust `bool`), `String` (owned Rust `String`), `Unit` (`()`).
- `let name: Type = expression` and `let mut name: Type = expression`.
- Assignments, `return`, `puts`, `if/else/end`, `while/end`.
- Integers, strings, booleans, function calls, parentheses, arithmetic, comparisons, boolean operations and unary `!`/`-`.
- `#` comments outside strings.

## Safety & Ownership model

Source code is processed through a modular compiler pipeline:
1. **Lexer (`src/lexer.rs`):** Emits span-aware tokens tracking byte offsets, lines, and columns.
2. **Parser (`src/parser.rs`):** Builds a typed Abstract Syntax Tree (`src/ast.rs`) via precedence-climbing.
3. **Semantic Analyzer (`src/analysis.rs`):** Validates the `main` entry point, duplicate definitions, undeclared variables, immutable assignments, parameter modes (`read`, `mut`, `take`), and tracks moves to prevent use-after-move prior to codegen.
4. **Diagnostics (`src/diagnostics.rs`):** Renders rustc-style source code snippets with aligned carets, column locations, and actionable hints.
5. **Codegen (`src/codegen.rs`):** Emits safe Rust emitting `#![forbid(unsafe_code)]`, `#![allow(unused_parens)]`, and `#![allow(unused_variables)]`, compiled via `rustc`.

Caller-site borrowing is callee-directed: passing a variable to a `read` parameter automatically borrows immutably; passing to a `mut` parameter automatically borrows mutably (asserting the variable was declared `mut`); passing to a `take` parameter transfers ownership and invalidates future uses.

## Milestones

- [x] **Milestone 1: Modular Pipeline & Native Diagnostics:** Span-aware lexer/AST, static validation, duplicate-name checks, immutable assignment checks, stdin piping (`-`), and source-mapped diagnostics.
- [x] **Milestone 2: Surface Types & Parameter Modes:** Implemented `read T`, `mut T`, `take T` parameter conventions (ADR 0001), inferred call-site borrowing without `&`/`&mut`, compile-time use-after-move detection, and ownership test suite.
- [x] **Milestone 3: Typed Analysis & Control-flow Checks:** Typed analysis, missing-return and unreachable-code detection, call-site borrow/exclusivity checks. (A standalone HIR/MIR and exhaustiveness checking are deferred: there are no `match`/enum constructs yet.)
- [x] **Milestone 4: Conformance & Benchmarks:** `benchmarks/run.sh` measures compile time, run time and peak RSS against equivalent Rust and Ruby programs. Sample (collatz, 300k): Opalescent 0.53s / 1.5 MB, Rust 0.52s / 1.5 MB, Ruby 2.44s / 28 MB.
- [x] **Milestone 5: Native Backend (exploration):** Decision recorded in [ADR 0002](adr/0002-native-backend-strategy.md): keep Rust emission; Cranelift/LLVM deferred.
