pub mod analysis;
pub mod ast;
pub mod codegen;
pub mod diagnostics;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;

pub use analysis::Analyzer;
pub use ast::Program;
pub use codegen::CodeGenerator;
pub use diagnostics::Diagnostic;
pub use lexer::Lexer;
pub use parser::Parser;
pub use span::Span;
pub use token::{Token, TokenKind};

pub fn compile(source: &str) -> Result<String, Diagnostic> {
    let tokens = Lexer::tokenize(source)?;
    let program = Parser::parse(tokens)?;
    Analyzer::analyze(&program)?;
    let rust_code = CodeGenerator::generate(&program);
    Ok(rust_code)
}
