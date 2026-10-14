use std::{env, fs, path::{Path, PathBuf}, process::{self, Command}, time::{SystemTime, UNIX_EPOCH}};

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("opal: {message}");
    process::exit(1)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String), Number(String), Str(String), Symbol(String),
}

fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() { chars.next(); continue; }
        if c == '#' { break; }
        if c == '"' {
            let mut value = String::from("\"");
            chars.next();
            let mut closed = false;
            while let Some(ch) = chars.next() {
                value.push(ch);
                if ch == '\\' {
                    let esc = chars.next().ok_or("unterminated escape")?;
                    if !matches!(esc, 'n' | 'r' | 't' | '\\' | '"') {
                        return Err(format!("unsupported string escape: \\{esc}"));
                    }
                    value.push(esc);
                } else if ch == '"' { closed = true; break; }
                else if ch.is_control() { return Err("control character in string".into()); }
            }
            if !closed { return Err("unterminated string".into()); }
            out.push(Token::Str(value));
        } else if c.is_ascii_alphabetic() || c == '_' {
            let mut s = String::new();
            while matches!(chars.peek(), Some(c) if c.is_ascii_alphanumeric() || *c == '_') {
                s.push(chars.next().unwrap());
            }
            out.push(Token::Ident(s));
        } else if c.is_ascii_digit() {
            let mut s = String::new();
            while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) { s.push(chars.next().unwrap()); }
            out.push(Token::Number(s));
        } else {
            chars.next();
            let mut op = c.to_string();
            if let Some(&next) = chars.peek() {
                if matches!((c, next), ('-', '>') | ('=', '=') | ('!', '=') | ('<', '=') | ('>', '=') | ('&', '&') | ('|', '|')) {
                    op.push(chars.next().unwrap());
                }
            }
            if !matches!(op.as_str(), "(" | ")" | "," | ":" | "=" | "+" | "-" | "*" | "/" | "%" | "<" | ">" | "==" | "!=" | "<=" | ">=" | "&&" | "||" | "!" | "->") {
                return Err(format!("unsupported token: {op:?}"));
            }
            out.push(Token::Symbol(op));
        }
    }
    Ok(out)
}
fn sym(t: &Token, s: &str) -> bool { matches!(t, Token::Symbol(v) if v == s) }
fn ident(t: &Token, s: &str) -> bool { matches!(t, Token::Ident(v) if v == s) }
fn name(t: &Token) -> Result<&str, String> {
    match t {
        Token::Ident(s) if !matches!(s.as_str(), "fn" | "let" | "mut" | "if" | "else" | "while" | "loop" | "match" | "use" | "mod" | "pub" | "unsafe" | "as" | "return" | "true" | "false" | "end" | "def" | "do" | "struct" | "impl" | "trait" | "enum" | "const" | "static" | "move" | "ref" | "self" | "Self" | "crate" | "super" | "type" | "where" | "async" | "await" | "extern") => Ok(s),
        _ => Err("expected non-reserved identifier".into())
    }
}
fn expect(t: &[Token], i: &mut usize, s: &str) -> Result<(), String> {
    if t.get(*i).is_some_and(|v| sym(v, s)) { *i += 1; Ok(()) }
    else { Err(format!("expected {s}")) }
}
fn ty(t: &Token) -> Result<&'static str, String> {
    match t {
        Token::Ident(s) => match s.as_str() {
            "Int" => Ok("i64"), "Bool" => Ok("bool"), "String" => Ok("String"), "Unit" => Ok("()"),
            _ => Err(format!("unknown type: {s}")),
        },
        _ => Err("expected type".into()),
    }
}
// Parses a restricted expression grammar, never embeds arbitrary source into generated Rust.
struct Expr<'a> { t: &'a [Token], i: usize }
impl<'a> Expr<'a> {
    fn parse(t: &'a [Token]) -> Result<String, String> {
        let mut p = Self { t, i: 0 };
        let s = p.binary(0)?;
        if p.i != t.len() { return Err("unexpected token after expression".into()); }
        Ok(s)
    }
    fn binary(&mut self, min: u8) -> Result<String, String> {
        let mut lhs = self.unary()?;
        loop {
            let Some(Token::Symbol(op)) = self.t.get(self.i) else { break };
            let prec = match op.as_str() {
                "||" => 1, "&&" => 2, "==" | "!=" => 3, "<" | ">" | "<=" | ">=" => 4,
                "+" | "-" => 5, "*" | "/" | "%" => 6, _ => 0,
            };
            if prec == 0 || prec < min { break; }
            let op = op.clone(); self.i += 1;
            let rhs = self.binary(prec + 1)?;
            lhs = format!("({lhs} {op} {rhs})");
        }
        Ok(lhs)
    }
    fn unary(&mut self) -> Result<String, String> {
        if self.t.get(self.i).is_some_and(|t| sym(t, "-") || sym(t, "!")) {
            let op = match &self.t[self.i] { Token::Symbol(s) => s.clone(), _ => unreachable!() };
            self.i += 1;
            return Ok(format!("({op}{})", self.unary()?));
        }
        self.primary()
    }
    fn primary(&mut self) -> Result<String, String> {
        let t = self.t.get(self.i).ok_or("expected expression")?;
        self.i += 1;
        match t {
            Token::Number(v) => Ok(format!("{v}i64")),
            Token::Str(v) => Ok(format!("{v}.to_string()")),
            Token::Ident(v) if v == "true" || v == "false" => Ok(v.clone()),
            Token::Ident(v) => {
                name(t)?;
                if self.t.get(self.i).is_some_and(|t| sym(t, "(")) {
                    self.i += 1;
                    let mut args = Vec::new();
                    if !self.t.get(self.i).is_some_and(|t| sym(t, ")")) {
                        loop {
                            args.push(self.binary(0)?);
                            if self.t.get(self.i).is_some_and(|t| sym(t, ",")) { self.i += 1; } else { break; }
                        }
                    }
                    expect(self.t, &mut self.i, ")")?;
                    Ok(format!("{v}({})", args.join(", ")))
                } else { Ok(v.clone()) }
            }
            Token::Symbol(s) if s == "(" => {
                let x = self.binary(0)?;
                expect(self.t, &mut self.i, ")")?;
                Ok(format!("({x})"))
            }
            _ => Err("invalid expression".into())
        }
    }
}
fn expr(t: &[Token]) -> Result<String, String> {
    if t.is_empty() { Err("missing expression".into()) } else { Expr::parse(t) }
}
fn assignment_pos(t: &[Token]) -> Option<usize> {
    let mut depth = 0;
    for (i, v) in t.iter().enumerate() {
        if sym(v, "(") { depth += 1; }
        if sym(v, ")") { depth -= 1; }
        if depth == 0 && sym(v, "=") { return Some(i); }
    }
    None
}
fn signature(t: &[Token]) -> Result<String, String> {
    // def name(arg: Type, ...) -> Type
    let function = name(t.get(1).ok_or("missing function name")?)?;
    let mut i = 2;
    expect(t, &mut i, "(")?;
    let mut params = Vec::new();
    if !t.get(i).is_some_and(|v| sym(v, ")")) {
        loop {
            let arg = name(t.get(i).ok_or("missing argument")?)?; i += 1;
            expect(t, &mut i, ":")?;
            let kind = ty(t.get(i).ok_or("missing argument type")?)?; i += 1;
            params.push(format!("{arg}: {kind}"));
            if t.get(i).is_some_and(|v| sym(v, ",")) { i += 1; } else { break; }
        }
    }
    expect(t, &mut i, ")")?;
    let ret = if t.get(i).is_some_and(|v| sym(v, "->")) {
        i += 1; let r = ty(t.get(i).ok_or("missing return type")?)?; i += 1; r
    } else { "()" };
    if i != t.len() { return Err("unexpected token in function declaration".into()); }
    Ok(format!("fn {function}({}) -> {ret} {{", params.join(", ")))
}
#[derive(Clone, Copy)]
enum Block { Function, If, While }
fn compile(src: &str) -> Result<String, String> {
    let mut functions = String::new();
    let mut main = String::from("fn main() {\n");
    let mut body = String::new();
    let mut stack: Vec<Block> = Vec::new();
    let mut in_function = false;
    for (index, line) in src.lines().enumerate() {
        let run = || -> Result<(), String> { Ok(()) };
        run()?;
        let t = lex(line).map_err(|e| format!("line {}: {e}", index + 1))?;
        if t.is_empty() { continue; }
        let parsed: Result<(), String> = (|| {
            let first = &t[0];
            if ident(first, "def") {
                if !stack.is_empty() || in_function { return Err("nested functions are not supported".into()); }
                let s = signature(&t)?;
                body.push_str(&s); body.push('\n');
                stack.push(Block::Function); in_function = true;
            } else if ident(first, "end") {
                if t.len() != 1 { return Err("end takes no arguments".into()); }
                let block = stack.pop().ok_or("unmatched end")?;
                body.push_str("}\n");
                if matches!(block, Block::Function) {
                    functions.push_str(&body); body.clear(); in_function = false;
                }
            } else if ident(first, "else") {
                if t.len() != 1 || !matches!(stack.last(), Some(Block::If)) { return Err("else without if".into()); }
                body.push_str("} else {\n");
            } else if ident(first, "if") || ident(first, "while") {
                if !in_function { return Err("control flow must be inside a function".into()); }
                let keyword = if ident(first, "if") { "if" } else { "while" };
                let condition = expr(&t[1..])?;
                body.push_str(&format!("{keyword} {condition} {{\n"));
                stack.push(if keyword == "if" { Block::If } else { Block::While });
            } else if ident(first, "let") {
                if !in_function { return Err("variables must be inside a function".into()); }
                let mut i = 1;
                let mutable = t.get(i).is_some_and(|v| ident(v, "mut"));
                if mutable { i += 1; }
                let variable = name(t.get(i).ok_or("missing variable name")?)?; i += 1;
                expect(&t, &mut i, ":")?;
                let kind = ty(t.get(i).ok_or("missing variable type")?)?; i += 1;
                expect(&t, &mut i, "=")?;
                let value = expr(&t[i..])?;
                body.push_str(&format!("let {}{variable}: {kind} = {value};\n", if mutable { "mut " } else { "" }));
            } else if ident(first, "return") {
                if !in_function { return Err("return outside function".into()); }
                body.push_str(&format!("return {};\n", expr(&t[1..])?));
            } else if ident(first, "puts") {
                if !in_function { return Err("puts must be inside a function".into()); }
                body.push_str(&format!("println!(\"{{}}\", {});\n", expr(&t[1..])?));
            } else if let Some(i) = assignment_pos(&t) {
                if !in_function { return Err("assignment outside function".into()); }
                if i != 1 { return Err("invalid assignment target".into()); }
                let variable = name(&t[0])?;
                body.push_str(&format!("{variable} = {};\n", expr(&t[2..])?));
            } else {
                if !in_function { return Err("only function declarations are allowed at top level".into()); }
                body.push_str(&format!("{};\n", expr(&t)?));
            }
            Ok(())
        })();
        parsed.map_err(|e| format!("line {}: {e}", index + 1))?;
    }
    if !stack.is_empty() { return Err("unclosed block: expected end".into()); }
    if !functions.contains("fn main(") { return Err("define def main() as the entry point".into()); }
    main.clear();
    Ok(format!("#![forbid(unsafe_code)]\n#![allow(unused_parens)]\n{functions}{main}"))
}
fn usage() -> ! {
    eprintln!("Usage: opal <check|build|run|emit-rust> <file.opal> [-o output]");
    process::exit(2);
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 { usage(); }
    let mode = args[1].as_str();
    if !matches!(mode, "check" | "build" | "run" | "emit-rust") { usage(); }
    let input = Path::new(&args[2]);
    let src = fs::read_to_string(input).unwrap_or_else(|e| fail(e));
    let rust = compile(&src).unwrap_or_else(|e| fail(e));
    if mode == "emit-rust" { print!("{rust}"); return; }
    let output = if let Some(i) = args.iter().position(|a| a == "-o") {
        PathBuf::from(args.get(i + 1).unwrap_or_else(|| usage()))
    } else {
        input.with_extension(if cfg!(windows) { "exe" } else { "" })
    };
    let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let temp = env::temp_dir().join(format!("opalescent-{}-{id}", process::id()));
    fs::create_dir(&temp).unwrap_or_else(|e| fail(e));
    let source = temp.join("generated.rs");
    fs::write(&source, rust).unwrap_or_else(|e| fail(e));
    let binary = if mode == "check" { temp.join("checked.rmeta") } else if mode == "run" { temp.join("program") } else { output.clone() };
    let mut command = Command::new("rustc");
    command.arg("--edition=2021").arg(&source);
    if mode == "check" { command.arg("--emit=metadata"); }
    else { command.arg("-C").arg("opt-level=3"); }
    let status = command.arg("-o").arg(&binary).status().unwrap_or_else(|e| fail(format!("rustc unavailable: {e}")));
    if !status.success() { let _ = fs::remove_dir_all(&temp); process::exit(status.code().unwrap_or(1)); }
    if mode == "run" {
        let code = Command::new(&binary).status().unwrap_or_else(|e| fail(e));
        let _ = fs::remove_dir_all(&temp);
        process::exit(code.code().unwrap_or(1));
    }
    let _ = fs::remove_dir_all(&temp);
    if mode == "build" { println!("Built {}", output.display()); } else { println!("Type check passed"); }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn basic() {
        let source = "def main()\nlet x: Int = 2 + 3 * 4\nputs x\nend";
        let generated = compile(source).unwrap();
        assert!(generated.contains("println!"));
        assert!(generated.contains("fn main()"));
    }
    #[test] fn rejects_injection() {
        assert!(compile("def main()\nputs 1; std::process::exit(1)\nend").is_err());
        assert!(compile("def main()\nputs \"unterminated\nend").is_err());
        assert!(compile("def main()\nif true\nend").is_err());
    }
    #[test] fn typed_functions() {
        let generated = compile("def add(a: Int, b: Int) -> Int\nreturn a + b\nend\ndef main()\nputs add(2, 3)\nend").unwrap();
        assert!(generated.contains("fn add(a: i64, b: i64) -> i64"));
    }
}
