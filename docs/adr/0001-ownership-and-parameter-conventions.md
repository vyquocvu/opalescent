# ADR 0001 — Ownership Foundation and Parameter Conventions (D001–D004)

- **Status:** DECIDED-PROVISIONAL
- **Deciders:** Opalescent Core Design
- **Date:** 2026-10-14
- **Resolves:** D001 (Ownership foundation), D002 (Parameter conventions), D003 (Caller move notation), D004 (Implicit cost policy)

---

## 1. Context

Opalescent’s core hypothesis is that an expressive, Ruby-readable language can provide statically enforceable memory safety and predictable native performance while transferring *incidental* ownership complexity from the programmer to the compiler.

In conventional systems languages like Rust, call sites require explicit syntactic decoration for borrows (`&val`, `&mut val`), even though the callee function signature already uniquely specifies the expected access capability. This ceremony increases visual noise and learning friction without adding new semantic information. Conversely, dynamically typed scripting languages (such as Ruby) and reference-counted languages (such as Swift or Python) often hide allocation, reference-counting traffic, or implicit cloning.

According to the Language Charter ([`docs/02-CHARTER.md`](../02-CHARTER.md)), Opalescent ranks **Soundness** (#1) and **Transparent Observable Costs** (#2) strictly above brevity.

---

## 2. Decision

### D001: Ownership Foundation
We adopt an **affine ownership model with static borrow checking**:
1. Every non-`Copy` owned value has exactly one logical owner at any point in program execution.
2. Passing an owned value by ownership (*consuming* it) invalidates the source place in the caller until explicit reinitialization.
3. Multiple concurrent shared readers OR a single exclusive mutable accessor are permitted; overlapping or conflicting live accesses to the same storage are rejected at compile time.
4. Borrow lifetimes cannot outlive the referenced storage; values are deterministically dropped when their owning scope exits.

### D002: Parameter Conventions
Function parameter signatures declare access capability using three explicit parameter modes:
- **`read T`**: Shared immutable borrow. The callee receives read-only access. Lowered to `&T` in generated code.
- **`mut T`**: Exclusive mutable borrow. The callee receives in-place mutation rights without taking ownership. Lowered to `&mut T` in generated code.
- **`take T`**: Ownership transfer. The callee consumes the value; the caller surrenders ownership. Lowered to owned `T` in generated code.

**Default Mode Convention:**
- For scalar `Copy` types (`Int`, `Bool`, `Unit`), omitting the mode defaults to value passing (equivalent in cost and semantics to `read`).
- For heap-allocated, non-`Copy` types (e.g. `String`), omitting the mode defaults to **`read T`** to optimize for low-ceremony reading without accidental moves. Explicit `take T` or `mut T` must be declared when consumption or mutation is intended.

### D003: Callee-Directed Caller Borrowing
Call sites omit explicit borrow operators (`&` or `&mut`) when the callee's parameter mode is statically declared:
- Calling `f(x)` where `f` expects `read T` automatically borrows `x` immutably.
- Calling `f(x)` where `f` expects `mut T` automatically borrows `x` mutably. The compiler verifies that `x` was bound with `let mut`.
- Calling `f(x)` where `f` expects `take T` moves `x` out of the caller place. Future accesses to `x` without reassignment are rejected at compile time.

### D004: Strict Cost Transparency Policy
The compiler will **never** insert an implicit `.clone()`, heap allocation, reference count increment, or synchronization lock merely to satisfy an ownership check. If an ownership conflict occurs, the compiler issues a compile-time error with an actionable diagnostic.

---

## 3. Options Considered & Rejected

1. **Rust-Style Call-Site Markers (`&x`, `&mut x`):**
   - *Rejected:* Callee signatures already encode the required parameter contract. Requiring callers to repeat this information introduces syntactic friction that runs counter to Opalescent’s Ruby-shaped readability goals.
2. **Swift/Valen-Style Implicit Copy on Escape:**
   - *Rejected:* Violates Principle #2 (Transparent Observable Costs). Implicit deep copying hides unpredictable performance cliffs and allocation hot-spots.
3. **Pure Mutable-Value Semantics (Hylo-style without explicit parameter annotations):**
   - *Rejected:* Without parameter mode annotations on public boundaries, whole-program escape analysis would be required, impairing local reasoning and separate compilation.

---

## 4. Code Examples

### Positive Examples
```ruby
def print_length(s: read String)
  puts s
end

def append_world(s: mut String)
  # in-place modification
end

def consume_string(s: take String)
  # takes ownership
end

def main()
  let mut message: String = "Hello"
  print_length(message)       # Inferred read borrow (&message)
  append_world(message)       # Inferred mut borrow (&mut message)
  consume_string(message)     # Moves message
end
```

### Negative (Compile-Fail) Examples

#### 1. Mutating an Immutable Binding via `mut T`
```ruby
def modify(s: mut String)
end

def main()
  let msg: String = "Immutable"
  modify(msg) # ERROR: cannot pass immutable variable `msg` to `mut` parameter
end
```

#### 2. Use-After-Move via `take T`
```ruby
def consume(s: take String)
end

def main()
  let msg: String = "Hello"
  consume(msg)
  puts msg    # ERROR: use of moved variable `msg`
end
```

---

## 5. Implementation Consequences

1. **Parser & AST (`src/parser.rs`, `src/ast.rs`):**
   - Support `read`, `mut`, and `take` as parameter mode prefixes in function signatures.
   - Associate `ParamMode` with each `Param` node.
2. **Semantic Analysis (`src/analysis.rs`):**
   - Maintain function signature parameter modes.
   - Validate call sites against callee parameter modes: verify mutable bindings for `mut T`, track move states for `take T`.
3. **Code Generation (`src/codegen.rs`):**
   - Map `read T` $\to$ `&T` (or `&str` / `&T`).
   - Map `mut T` $\to$ `&mut T`.
   - Map `take T` $\to$ `T`.
   - Map call arguments based on callee mode: `&arg`, `&mut arg`, or `arg`.

---

## 6. Test Obligations
- Conformance test validating seamless `read String` passing without caller `&`.
- Conformance test verifying `mut String` in-place modification.
- Compile-fail test verifying rejection of immutable variables passed to `mut` parameters.
- Compile-fail test verifying rejection of use-after-move on `take` parameters.
