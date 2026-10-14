use std::collections::HashMap;

use crate::ast::*;
use crate::diagnostics::Diagnostic;
use crate::span::Span;

#[derive(Clone)]
struct VarInfo {
    mutable: bool,
    span: Span,
}

struct Scope {
    variables: HashMap<String, VarInfo>,
}

pub struct Analyzer<'a> {
    program: &'a Program,
    function_signatures: HashMap<String, (usize, Span)>,
    scopes: Vec<Scope>,
}

impl<'a> Analyzer<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self {
            program,
            function_signatures: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    pub fn analyze(program: &'a Program) -> Result<(), Diagnostic> {
        let mut analyzer = Analyzer::new(program);
        analyzer.run()
    }

    fn run(&mut self) -> Result<(), Diagnostic> {
        // Collect function signatures and check for duplicates
        for func in &self.program.functions {
            if let Some((_, first_span)) = self.function_signatures.get(&func.name) {
                return Err(Diagnostic::error(
                    format!("duplicate function definition: `{}`", func.name),
                    func.span,
                )
                .with_hint(format!(
                    "first defined at line {}:{}",
                    first_span.line, first_span.col
                )));
            }
            self.function_signatures
                .insert(func.name.clone(), (func.params.len(), func.span));
        }

        // Validate entry point: def main()
        let main_fn = self
            .program
            .functions
            .iter()
            .find(|f| f.name == "main")
            .ok_or_else(|| {
                Diagnostic::error(
                    "missing entry point: define `def main()` as the program entry point",
                    Span::dummy(),
                )
            })?;

        if !main_fn.params.is_empty() {
            return Err(Diagnostic::error(
                "entry point `main` cannot take parameters",
                main_fn.span,
            ));
        }

        // Validate each function body
        for func in &self.program.functions {
            self.analyze_function(func)?;
        }

        Ok(())
    }

    fn analyze_function(&mut self, func: &FunctionDecl) -> Result<(), Diagnostic> {
        self.scopes.clear();
        self.push_scope();

        // Check for duplicate parameter names and insert them into scope
        for param in &func.params {
            if self.lookup_local(&param.name).is_some() {
                return Err(Diagnostic::error(
                    format!("duplicate parameter name: `{}`", param.name),
                    param.span,
                ));
            }
            self.insert_var(
                param.name.clone(),
                VarInfo {
                    mutable: false,
                    span: param.span,
                },
            );
        }

        // Validate statements in function body
        for stmt in &func.body {
            self.analyze_stmt(stmt)?;
        }

        self.pop_scope();
        Ok(())
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<(), Diagnostic> {
        match stmt {
            Stmt::Let {
                mutable,
                name,
                value,
                span,
                ..
            } => {
                self.analyze_expr(value)?;
                self.insert_var(
                    name.clone(),
                    VarInfo {
                        mutable: *mutable,
                        span: *span,
                    },
                );
            }
            Stmt::Assign {
                target,
                value,
                span,
            } => {
                self.analyze_expr(value)?;
                let var = self.lookup_var(target).ok_or_else(|| {
                    Diagnostic::error(format!("cannot assign to undefined variable `{target}`"), *span)
                })?;

                if !var.mutable {
                    return Err(Diagnostic::error(
                        format!("cannot assign to immutable variable `{target}`"),
                        *span,
                    )
                    .with_hint(format!(
                        "`{target}` was defined as immutable at line {}:{}; consider declaring as mutable: `let mut {target}`",
                        var.span.line, var.span.col
                    )));
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.analyze_expr(condition)?;
                self.push_scope();
                for s in then_branch {
                    self.analyze_stmt(s)?;
                }
                self.pop_scope();

                if let Some(else_stmts) = else_branch {
                    self.push_scope();
                    for s in else_stmts {
                        self.analyze_stmt(s)?;
                    }
                    self.pop_scope();
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.analyze_expr(condition)?;
                self.push_scope();
                for s in body {
                    self.analyze_stmt(s)?;
                }
                self.pop_scope();
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.analyze_expr(expr)?;
                }
            }
            Stmt::Puts { value, .. } => {
                self.analyze_expr(value)?;
            }
            Stmt::Expr(expr) => {
                self.analyze_expr(expr)?;
            }
        }
        Ok(())
    }

    fn analyze_expr(&self, expr: &Expr) -> Result<(), Diagnostic> {
        match &expr.kind {
            ExprKind::IntLit(_) | ExprKind::BoolLit(_) | ExprKind::StrLit(_) => Ok(()),
            ExprKind::Variable(name) => {
                if self.lookup_var(name).is_none() {
                    return Err(Diagnostic::error(
                        format!("undefined variable `{name}`"),
                        expr.span,
                    ));
                }
                Ok(())
            }
            ExprKind::Unary { expr: inner, .. } => self.analyze_expr(inner),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.analyze_expr(lhs)?;
                self.analyze_expr(rhs)?;
                Ok(())
            }
            ExprKind::Call { callee, args } => {
                for arg in args {
                    self.analyze_expr(arg)?;
                }
                let &(expected_params, _) = self
                    .function_signatures
                    .get(callee)
                    .ok_or_else(|| {
                        Diagnostic::error(
                            format!("call to undefined function `{callee}`"),
                            expr.span,
                        )
                    })?;

                if args.len() != expected_params {
                    return Err(Diagnostic::error(
                        format!(
                            "function `{callee}` expects {} argument(s), but {} were provided",
                            expected_params,
                            args.len()
                        ),
                        expr.span,
                    ));
                }
                Ok(())
            }
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            variables: HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_var(&mut self, name: String, info: VarInfo) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.variables.insert(name, info);
        }
    }

    fn lookup_local(&self, name: &str) -> Option<&VarInfo> {
        self.scopes.last().and_then(|s| s.variables.get(name))
    }

    fn lookup_var(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.variables.get(name) {
                return Some(info);
            }
        }
        None
    }
}
