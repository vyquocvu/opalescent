use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

use opalescent::compile;

fn usage() -> ! {
    eprintln!("Usage: opal <check|build|run|emit-rust> <file.opal> [-o output]");
    process::exit(2);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        usage();
    }

    let mode = args[1].as_str();
    if !matches!(mode, "check" | "build" | "run" | "emit-rust") {
        usage();
    }

    let (src, file_display, default_output) = if args[2] == "-" {
        use std::io::Read;
        let mut buffer = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
            eprintln!("opal: failed to read standard input: {e}");
            process::exit(1);
        }
        (
            buffer,
            "<stdin>".to_string(),
            PathBuf::from(if cfg!(windows) { "out.exe" } else { "out" }),
        )
    } else {
        let input = Path::new(&args[2]);
        let content = match fs::read_to_string(input) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("opal: failed to read file `{}`: {e}", input.display());
                process::exit(1);
            }
        };
        (
            content,
            input.display().to_string(),
            input.with_extension(if cfg!(windows) { "exe" } else { "" }),
        )
    };

    let rust = match compile(&src) {
        Ok(code) => code,
        Err(diag) => {
            eprintln!("{}", diag.render(&src, &file_display));
            process::exit(1);
        }
    };

    if mode == "emit-rust" {
        print!("{rust}");
        return;
    }

    let output = if let Some(i) = args.iter().position(|a| a == "-o") {
        PathBuf::from(args.get(i + 1).unwrap_or_else(|| usage()))
    } else {
        default_output
    };

    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = env::temp_dir().join(format!("opalescent-{}-{id}", process::id()));
    if let Err(e) = fs::create_dir(&temp) {
        eprintln!("opal: failed to create temporary directory: {e}");
        process::exit(1);
    }

    let source = temp.join("generated.rs");
    if let Err(e) = fs::write(&source, &rust) {
        let _ = fs::remove_dir_all(&temp);
        eprintln!("opal: failed to write generated code: {e}");
        process::exit(1);
    }

    let binary = if mode == "check" {
        temp.join("checked.rmeta")
    } else if mode == "run" {
        temp.join("program")
    } else {
        output.clone()
    };

    let mut command = Command::new("rustc");
    command.arg("--edition=2021").arg(&source);
    if mode == "check" {
        command.arg("--emit=metadata");
    } else {
        command.arg("-C").arg("opt-level=3");
    }

    let status = match command.arg("-o").arg(&binary).status() {
        Ok(s) => s,
        Err(e) => {
            let _ = fs::remove_dir_all(&temp);
            eprintln!("opal: rustc unavailable: {e}");
            process::exit(1);
        }
    };

    if !status.success() {
        let _ = fs::remove_dir_all(&temp);
        process::exit(status.code().unwrap_or(1));
    }

    if mode == "run" {
        let code = match Command::new(&binary).status() {
            Ok(c) => c,
            Err(e) => {
                let _ = fs::remove_dir_all(&temp);
                eprintln!("opal: execution failed: {e}");
                process::exit(1);
            }
        };
        let _ = fs::remove_dir_all(&temp);
        process::exit(code.code().unwrap_or(1));
    }

    let _ = fs::remove_dir_all(&temp);
    if mode == "build" {
        println!("Built {}", output.display());
    } else {
        println!("Type check passed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let source = "def main()\nlet x: Int = 2 + 3 * 4\nputs x\nend";
        let generated = compile(source).unwrap();
        assert!(generated.contains("println!"));
        assert!(generated.contains("fn main() -> ()"));
    }

    #[test]
    fn typed_functions() {
        let source = "def add(a: Int, b: Int) -> Int\nreturn a + b\nend\ndef main()\nputs add(2, 3)\nend";
        let generated = compile(source).unwrap();
        assert!(generated.contains("fn add(a: i64, b: i64) -> i64"));
    }

    #[test]
    fn rejects_injection() {
        assert!(compile("def main()\nputs 1; std::process::exit(1)\nend").is_err());
        assert!(compile("def main()\nputs \"unterminated\nend").is_err());
        assert!(compile("def main()\nif true\nend").is_err());
    }

    #[test]
    fn rejects_duplicate_function() {
        let source = "def foo() -> Unit\nend\ndef foo() -> Unit\nend\ndef main()\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("duplicate function definition: `foo`"));
        assert_eq!(err.span.line, 3);
    }

    #[test]
    fn rejects_duplicate_parameter() {
        let source = "def foo(x: Int, x: Int) -> Unit\nend\ndef main()\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("duplicate parameter name: `x`"));
    }

    #[test]
    fn rejects_immutable_assignment() {
        let source = "def main()\nlet x: Int = 1\nx = 2\nputs x\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("cannot assign to immutable variable `x`"));
        assert_eq!(err.span.line, 3);
    }

    #[test]
    fn allows_mutable_assignment() {
        let source = "def main()\nlet mut x: Int = 1\nx = 2\nputs x\nend";
        let generated = compile(source).unwrap();
        assert!(generated.contains("let mut x: i64 = 1i64;"));
    }

    #[test]
    fn rejects_undefined_variable() {
        let source = "def main()\nputs y\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("undefined variable `y`"));
        assert_eq!(err.span.line, 2);
    }

    #[test]
    fn rejects_undefined_function() {
        let source = "def main()\nunknown_func()\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("call to undefined function `unknown_func`"));
    }

    #[test]
    fn rejects_arity_mismatch() {
        let source = "def add(a: Int, b: Int) -> Int\nreturn a + b\nend\ndef main()\nputs add(1)\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("function `add` expects 2 argument(s), but 1 were provided"));
    }

    #[test]
    fn diagnostic_rendering() {
        let source = "def main()\nlet x: Int = 10\nx = 20\nend";
        let err = compile(source).unwrap_err();
        let rendered = err.render(source, "test.opal");
        assert!(rendered.contains("error: cannot assign to immutable variable `x`"));
        assert!(rendered.contains("test.opal:3:1"));
        assert!(rendered.contains("x = 20"));
        assert!(rendered.contains("consider declaring as mutable: `let mut x`"));
    }

    #[test]
    fn read_parameter_mode() {
        let source = "def print_str(s: read String) -> Unit\nputs s\nend\ndef main()\nlet msg: String = \"hi\"\nprint_str(msg)\nend";
        let generated = compile(source).unwrap();
        assert!(generated.contains("fn print_str(s: &str) -> ()"));
        assert!(generated.contains("print_str(&msg)"));
    }

    #[test]
    fn mut_parameter_mode() {
        let source = "def update(s: mut String) -> Unit\nend\ndef main()\nlet mut msg: String = \"hi\"\nupdate(msg)\nend";
        let generated = compile(source).unwrap();
        assert!(generated.contains("fn update(s: &mut String) -> ()"));
        assert!(generated.contains("update(&mut msg)"));
    }

    #[test]
    fn rejects_immutable_to_mut_param() {
        let source = "def update(s: mut String) -> Unit\nend\ndef main()\nlet msg: String = \"hi\"\nupdate(msg)\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("cannot pass immutable variable `msg` as mutable argument"));
    }

    #[test]
    fn rejects_use_after_move() {
        let source = "def consume(s: take String) -> Unit\nend\ndef main()\nlet msg: String = \"hi\"\nconsume(msg)\nputs msg\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("use of moved variable `msg`"));
    }

    #[test]
    fn rejects_type_mismatch_binding() {
        let source = "def main()\nlet x: Int = true\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("type mismatch in binding"));
    }

    #[test]
    fn rejects_non_bool_condition() {
        let source = "def main()\nif 1\nputs 1\nend\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("must be `Bool`"));
    }

    #[test]
    fn rejects_missing_return_path() {
        let source = "def f(x: Int) -> Int\nif x < 0\nreturn 0\nend\nend\ndef main()\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("not all control paths return a value"));
    }

    #[test]
    fn rejects_unreachable_after_return() {
        let source = "def main()\nreturn\nputs 1\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("unreachable code"));
    }

    #[test]
    fn rejects_unreachable_constant_branch() {
        let source = "def main()\nif true\nputs 1\nelse\nputs 2\nend\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("unreachable code"));
    }


    #[test]
    fn rejects_call_exclusivity() {
        let source = "def f(a: mut Int, b: mut Int) -> Unit\nend\ndef main()\nlet mut x: Int = 1\nf(x, x)\nend";
        let err = compile(source).unwrap_err();
        assert!(err.message.contains("while it is also used in another argument"));
    }
}
