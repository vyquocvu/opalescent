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

## Safety model

Source is tokenized and parsed into a restricted expression grammar. It emits `#![forbid(unsafe_code)]` and compiles through `rustc`. Rust type checking, borrow checking, and ownership apply to the generated program. Mutable variables require `mut`. This MVP does **not** expose explicit borrowing/reference syntax, user-defined structs, generics, modules, IO or FFI. It does not claim Rust-level safety for future language features without dedicated validation. Integer overflow behavior follows the compilation profile; future releases must specify it explicitly. Compiler-generated errors currently reference generated Rust lines.

## Next milestones

1. Introduce span-aware lexer/AST and diagnostic source maps; enforce declaration and duplicate-name checks.
2. Add `&T`, `&mut T`, moves, lifetimes via an explicit surface type system; compile-fail ownership tests.
3. Typed HIR/MIR with control-flow analysis and exhaustive tests, and a documented standard library.
4. Benchmark optimized native binaries against Rust; fuzz parser, build reproducibility, and harden CLI.
5. Consider LLVM/Cranelift backend only after language semantics are stable.
