# 03 — Surface syntax proposals

**Status: CANDIDATE**, examples are sketches, not runnable promises.

```ruby
def greet(name: String) -> String
  "Hello, #{name}"
end

def main()
  let user = "Ada"
  puts greet(user)
end
```

## Draft lexical goals
- UTF-8 source. Keywords and identifier Unicode policy OPEN (security/confusables).
- Significant `end` blocks. Newline as statement separator except in delimited expressions. Semicolons may be excluded.
- `#` comments, Ruby-shaped calls and interpolation only if statically typed and unambiguous.
- No ambient monkey patching or `eval` in safe core.
- Operator precedence and whitespace/newline continuation must be fully enumerated in normative grammar; examples alone are insufficient.

## Syntax design alternatives to decide
- Binding: `let x`/`let mut x` vs `x =`/`var x =`. The latter needs declaration vs reassignment disambiguation.
- Function: `def f(x: T) -> U` vs an explicit parameter-mode syntax.
- Predicate method `empty?`, bang method `save!`: lexer grammar and punctuation conflict resolution.
- Blocks: `do ... end` closures vs `{ ... }`, capture kinds and closure escape.
- Calls with and without parentheses, and disambiguation from local variables.
- Pattern matching, generics and traits: defer concrete syntax pending type model.

## Syntax freeze requirements
A formal EBNF/PEG, precedence table, lexical edge-case matrix, format specification, unambiguous parse tree examples, grammar regression corpus and a migration policy.
