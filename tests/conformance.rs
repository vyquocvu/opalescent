//! Conformance suite: compile-fail fixtures with expected message + span line,
//! positive programs, and example compilation (docs/08-CONFORMANCE.md).

use opalescent::compile;

/// (name, source, expected message fragment, expected 1-based line)
const FAIL: &[(&str, &str, &str, usize)] = &[
    ("missing_main", "def f() -> Unit\nend", "main", 1),
    ("undefined_var", "def main()\nputs y\nend", "undefined variable `y`", 2),
    ("immutable_assign", "def main()\nlet x: Int = 1\nx = 2\nend", "cannot assign to immutable variable `x`", 3),
    ("binding_mismatch", "def main()\nlet x: Int = true\nend", "type mismatch in binding", 2),
    ("non_bool_cond", "def main()\nif 1\nputs 1\nend\nend", "must be `Bool`", 2),
    ("while_non_bool", "def main()\nwhile 1\nputs 1\nend\nend", "must be `Bool`", 2),
    ("missing_return", "def f(x: Int) -> Int\nif x < 0\nreturn 0\nend\nend\ndef main()\nend", "not all control paths return a value", 1),
    ("unreachable", "def main()\nreturn\nputs 1\nend", "unreachable code", 3),
    ("arity", "def f(a: Int) -> Int\nreturn a\nend\ndef main()\nputs f(1, 2)\nend", "expects 1 argument(s)", 5),
    ("undefined_fn", "def main()\nnope()\nend", "call to undefined function `nope`", 2),
    ("dup_fn", "def f() -> Unit\nend\ndef f() -> Unit\nend\ndef main()\nend", "duplicate function definition: `f`", 3),
    ("dup_param", "def f(x: Int, x: Int) -> Unit\nend\ndef main()\nend", "duplicate parameter name: `x`", 1),
    ("use_after_move", "def c(s: take String) -> Unit\nend\ndef main()\nlet m: String = \"a\"\nc(m)\nputs m\nend", "use of moved variable `m`", 6),
    ("immutable_to_mut", "def u(s: mut String) -> Unit\nend\ndef main()\nlet m: String = \"a\"\nu(m)\nend", "cannot pass immutable variable `m` as mutable argument", 5),
    ("exclusivity", "def f(a: mut Int, b: mut Int) -> Unit\nend\ndef main()\nlet mut x: Int = 1\nf(x, x)\nend", "while it is also used in another argument", 5),
    ("unterminated_string", "def main()\nputs \"abc\nend", "", 2),
    ("unclosed_if", "def main()\nif true\nend", "", 0),
    ("injection", "def main()\nputs 1; std::process::exit(1)\nend", "", 2),
    ("dup_enum", "enum C\nA\nend\nenum C\nB\nend\ndef main()\nend", "duplicate enum definition: `C`", 4),
    ("empty_enum", "enum C\nend\ndef main()\nend", "must have at least one variant", 1),
    ("dup_variant", "enum C\nA\nA\nend\ndef main()\nend", "duplicate variant `A` in enum `C`", 3),
    ("unknown_type", "enum C\nA\nend\ndef main()\nlet x: Shade = C.A\nend", "unknown type `Shade`", 5),
    ("unknown_param_type", "def f(x: Shade) -> Int\nreturn 1\nend\ndef main()\nend", "unknown type `Shade`", 1),
    ("unknown_enum_expr", "def main()\nlet x: Int = Shade.Red\nend", "unknown enum `Shade`", 2),
    ("unknown_variant", "enum C\nA\nend\ndef main()\nlet x: C = C.B\nend", "enum `C` has no variant `B`", 5),
    ("match_non_enum", "def main()\nlet x: Int = 1\nmatch x\nwhen C.A\nputs 1\nend\nend", "`match` scrutinee must be an enum", 3),
    ("match_pattern_enum_mismatch", "enum C\nA\nend\nenum D\nB\nend\ndef main()\nlet x: C = C.A\nmatch x\nwhen D.B\nputs 1\nend\nend", "does not match scrutinee type", 10),
    ("match_unknown_variant", "enum C\nA\nend\ndef main()\nlet x: C = C.A\nmatch x\nwhen C.Z\nputs 1\nend\nend", "has no variant `Z`", 7),
    ("non_exhaustive_match", "enum C\nA\nB\nend\ndef main()\nlet x: C = C.A\nmatch x\nwhen C.A\nputs 1\nend\nend", "non-exhaustive `match`: missing `C.B`", 7),
    ("unreachable_pattern", "enum C\nA\nB\nend\ndef main()\nlet x: C = C.A\nmatch x\nwhen C.A\nputs 1\nwhen C.A\nputs 2\nend\nend", "unreachable pattern", 10),
    ("unreachable_match_else", "enum C\nA\nend\ndef main()\nlet x: C = C.A\nmatch x\nwhen C.A\nputs 1\nelse\nputs 2\nend\nend", "unreachable `else`", 10),
    ("match_no_arms", "enum C\nA\nend\ndef main()\nlet x: C = C.A\nmatch x\nend\nend", "must have at least one `when` arm or an `else` branch", 6),
    ("puts_enum", "enum C\nA\nend\ndef main()\nputs C.A\nend", "must be printable", 5),
    ("puts_not_an_identifier", "def puts() -> Unit\nend\ndef main()\nend", "expected identifier, found `puts`", 1),
];

#[test]
fn compile_fail_fixtures() {
    for (name, src, msg, line) in FAIL {
        let err = match compile(src) {
            Ok(_) => panic!("{name}: expected a compile error"),
            Err(e) => e,
        };
        assert!(
            err.message.contains(msg),
            "{name}: message `{}` lacks `{msg}`",
            err.message
        );
        if *line != 0 {
            assert_eq!(err.span.line, *line, "{name}: wrong span line");
        }
        // Rendering must never panic and must mention the line content.
        let rendered = err.render(src, "t.opal");
        assert!(rendered.contains("error:"), "{name}: bad render");
    }
}

/// Positive corpus: (name, source, substrings expected in generated Rust)
const PASS: &[(&str, &str, &[&str])] = &[
    ("hello", "def main()\nputs \"hi\"\nend", &["println!", "#![forbid(unsafe_code)]"]),
    ("arith", "def main()\nputs 1 + 2 * 3 - 4 / 2\nend", &["println!"]),
    ("shadow_ok", "def main()\nlet mut a: Int = 1\na = a + 1\nputs a\nend", &["let mut a: i64"]),
    ("recursion", "def f(n: Int) -> Int\nif n <= 1\nreturn 1\nend\nreturn n * f(n - 1)\nend\ndef main()\nputs f(5)\nend", &["fn f(n: i64) -> i64"]),
    ("read_mode", "def p(s: read String) -> Unit\nputs s\nend\ndef main()\nlet m: String = \"a\"\np(m)\np(m)\nend", &["&str", "p(&m)"]),
    ("take_mode", "def c(s: take String) -> Unit\nend\ndef main()\nlet m: String = \"a\"\nc(m)\nend", &["fn c(s: String)"]),
    ("bool_ops", "def main()\nputs !false && (1 < 2 || 3 == 4)\nend", &["println!"]),
    ("neg", "def main()\nputs -5\nend", &["println!"]),
    ("comment", "# c\ndef main()\nputs 1 # trailing\nend", &["println!"]),
    (
        "enum_match",
        "enum Color\nRed\nGreen\nBlue\nUnknown\nend\ndef describe(c: read Color) -> String\nmatch c\nwhen Color.Red\nreturn \"red\"\nwhen Color.Green, Color.Blue\nreturn \"cool\"\nelse\nreturn \"other\"\nend\nend\ndef main()\nputs describe(Color.Green)\nend",
        &["enum Color", "Color::Green | Color::Blue", "match c", "_ =>"],
    ),
    (
        "enum_exhaustive",
        "enum Color\nRed\nGreen\nend\ndef main()\nlet c: Color = Color.Red\nmatch c\nwhen Color.Red\nputs 1\nwhen Color.Green\nputs 2\nend\nend",
        &["enum Color", "Color::Red =>", "let c: Color = Color::Red"],
    ),
];

#[test]
fn positive_corpus() {
    for (name, src, needles) in PASS {
        let out = compile(src).unwrap_or_else(|e| panic!("{name}: {}", e.message));
        for n in *needles {
            assert!(out.contains(n), "{name}: generated code lacks `{n}`");
        }
        // Compilation must be deterministic.
        assert_eq!(out, compile(src).unwrap(), "{name}: nondeterministic output");
    }
}

#[test]
fn examples_compile() {
    for entry in std::fs::read_dir("examples").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "opal") {
            let src = std::fs::read_to_string(&path).unwrap();
            compile(&src).unwrap_or_else(|e| panic!("{}: {}", path.display(), e.message));
        }
    }
}

#[test]
fn parser_never_panics_on_garbage() {
    let inputs = [
        "", "def", "def main(", "def main()\n", "end end end", "\"", "#", "1 +", "let", "def main()\nlet x:\nend",
        "def main()\nwhile\nend", "💥", "def main() -> \nend", "def f(x: take) -> Unit\nend",
    ];
    for i in inputs {
        let _ = compile(i); // must return Ok/Err, never panic
    }
}
