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
