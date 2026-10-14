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
- `Int` (Rust `i64`), `Bool` (Rust `bool`), `String` (owned Rust `String`), `Unit` (`()`).
- `let name: Type = expression` and `let mut name: Type = expression`.
- Assignments, `return`, `puts`, `if/else/end`, `while/end`.
- Integers, strings, booleans, function calls, parentheses, arithmetic, comparisons, boolean operations and unary `!`/`-`.
- `#` comments outside strings.

## Safety & Diagnostic model

Source code is processed through a modular compiler pipeline:
1. **Lexer (`src/lexer.rs`):** Emits span-aware tokens tracking byte offsets, lines, and columns.
2. **Parser (`src/parser.rs`):** Builds a typed Abstract Syntax Tree (`src/ast.rs`) via precedence-climbing.
3. **Semantic Analyzer (`src/analysis.rs`):** Validates the `main` entry point, duplicate definitions, undeclared variables, immutable assignments, and call arity prior to codegen.
4. **Diagnostics (`src/diagnostics.rs`):** Renders rustc-style source code snippets with aligned carets, column locations, and actionable hints.
5. **Codegen (`src/codegen.rs`):** Emits safe Rust emitting `#![forbid(unsafe_code)]` and `#![allow(unused_parens)]`, compiled via `rustc`.

This MVP does **not** expose explicit borrowing/reference syntax, user-defined structs, generics, modules, IO or FFI. It does not claim Rust-level safety for future language features without dedicated validation. Integer overflow behavior follows the compilation profile.

## Milestones

- [x] **Milestone 1: Modular Pipeline & Native Diagnostics:** Span-aware lexer/AST, static validation, duplicate-name checks, immutable assignment checks, stdin piping (`-`), and source-mapped diagnostics.
- [ ] **Milestone 2: Surface Types & Parameter Modes:** Introduce `read T` / `mut T` parameter conventions, borrowing syntax, and compile-fail ownership test fixtures.
- [ ] **Milestone 3: Typed HIR/MIR & Control-flow Analysis:** Complete control-flow graph, dead-code detection, and exhaustiveness checking.
- [ ] **Milestone 4: Conformance & Benchmarks:** Measure throughput, peak RSS, and compile times against equivalent Rust and Ruby implementations.
- [ ] **Milestone 5: Native Backend:** Explore LLVM / Cranelift codegen once language semantics stabilize.
