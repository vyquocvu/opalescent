use std::collections::HashMap;

use crate::ast::*;
use crate::diagnostics::Diagnostic;
use crate::span::Span;

#[derive(Clone)]
struct VarInfo {
    mutable: bool,
    ty: TypeKind,
    moved: bool,
    span: Span,
}

struct Scope {
    variables: HashMap<String, VarInfo>,
}

pub struct Analyzer<'a> {
    program: &'a Program,
    function_signatures: HashMap<String, (Vec<(ParamMode, TypeKind)>, TypeKind, Span)>,
    scopes: Vec<Scope>,
    current_return_type: TypeKind,
}

impl<'a> Analyzer<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self {
            program,
            function_signatures: HashMap::new(),
            scopes: Vec::new(),
            current_return_type: TypeKind::Unit,
        }
    }

    pub fn analyze(program: &'a Program) -> Result<(), Diagnostic> {
        let mut analyzer = Analyzer::new(program);
        analyzer.run()
    }

    fn run(&mut self) -> Result<(), Diagnostic> {
        for func in &self.program.functions {
            if let Some((_, _, first_span)) = self.function_signatures.get(&func.name) {
                return Err(Diagnostic::error(
                    format!("duplicate function definition: `{}`", func.name),
                    func.span,
                )
                .with_hint(format!(
                    "first defined at line {}:{}",
                    first_span.line, first_span.col
                )));
            }
            let sig = func.params.iter().map(|p| (p.mode, p.ty.kind)).collect();
            self.function_signatures
                .insert(func.name.clone(), (sig, func.ret_ty.kind, func.span));
        }

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

        for func in &self.program.functions {
            self.analyze_function(func)?;
        }

        Ok(())
    }

    fn analyze_function(&mut self, func: &FunctionDecl) -> Result<(), Diagnostic> {
        self.scopes.clear();
        self.push_scope();
        self.current_return_type = func.ret_ty.kind;

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
                    mutable: param.mode == ParamMode::Mut,
                    ty: param.ty.kind,
                    moved: false,
                    span: param.span,
                },
            );
        }

        let returns = self.analyze_block(&func.body)?;
        if func.ret_ty.kind != TypeKind::Unit && !returns {
            return Err(Diagnostic::error(
                "not all control paths return a value",
                func.span,
            ));
        }

        self.pop_scope();
        Ok(())
    }

    fn analyze_block(&mut self, stmts: &[Stmt]) -> Result<bool, Diagnostic> {
        let mut terminated = false;
        let mut last_returns = false;
        for stmt in stmts {
            if terminated {
                return Err(Diagnostic::error("unreachable code", stmt.span()));
            }
            last_returns = self.analyze_stmt(stmt)?;
            if matches!(stmt, Stmt::Return { .. }) {
                terminated = true;
            }
        }
        Ok(last_returns)
    }

    #[allow(unused_assignments)]
    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<bool, Diagnostic> {
        let mut result = false;
        match stmt {
            Stmt::Let {
                mutable,
                name,
                ty,
                value,
                span,
            } => {
                let value_ty = self.expr_type(value)?;
                if value_ty != ty.kind {
                    return Err(Diagnostic::error(
                        format!(
                            "type mismatch in binding `{name}`: expected `{}`, found `{}`",
                            ty.kind.rust_type_name(),
                            value_ty.rust_type_name()
                        ),
                        value.span,
                    ));
                }
                self.insert_var(
                    name.clone(),
                    VarInfo {
                        mutable: *mutable,
                        ty: ty.kind,
                        moved: false,
                        span: *span,
                    },
                );
                result = false;
            }
            Stmt::Assign {
                target,
                value,
                span,
            } => {
                let value_ty = self.expr_type(value)?;
                let var = self.lookup_var_mut(target).ok_or_else(|| {
                    Diagnostic::error(format!("cannot assign to undefined variable `{target}`"), *span)
                })?;

                if !var.mutable {
                    let var_span = var.span;
                    return Err(Diagnostic::error(
                        format!("cannot assign to immutable variable `{target}`"),
                        *span,
                    )
                    .with_hint(format!(
                        "`{target}` was defined as immutable at line {}:{}; consider declaring as mutable: `let mut {target}`",
                        var_span.line, var_span.col
                    )));
                }

                if var.ty != value_ty {
                    let expected = var.ty;
                    return Err(Diagnostic::error(
                        format!(
                            "type mismatch in assignment to `{target}`: expected `{}`, found `{}`",
                            expected.rust_type_name(),
                            value_ty.rust_type_name()
                        ),
                        value.span,
                    ));
                }

                var.moved = false;
                result = false;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_ty = self.expr_type(condition)?;
                if cond_ty != TypeKind::Bool {
                    return Err(Diagnostic::error(
                        format!(
                            "condition in `if` must be `Bool`, found `{}`",
                            cond_ty.rust_type_name()
                        ),
                        condition.span,
                    ));
                }
                self.push_scope();
                let cond_is_true = matches!(condition.kind, ExprKind::BoolLit(true));
                let cond_is_false = matches!(condition.kind, ExprKind::BoolLit(false));

                let then_ret = if cond_is_false {
                    if let Some(stmt) = then_branch.first() {
                        return Err(Diagnostic::error("unreachable code", stmt.span()));
                    }
                    true
                } else {
                    self.analyze_block(then_branch)?
                };
                self.pop_scope();

                let else_ret = if let Some(else_stmts) = else_branch {
                    self.push_scope();
                    let r = if cond_is_true {
                        if let Some(stmt) = else_stmts.first() {
                            return Err(Diagnostic::error("unreachable code", stmt.span()));
                        }
                        Some(true)
                    } else {
                        Some(self.analyze_block(else_stmts)?)
                    };
                    self.pop_scope();
                    r
                } else {
                    None
                };

                let all_ret = if cond_is_true {
                    then_ret
                } else if cond_is_false {
                    else_ret.unwrap_or(false)
                } else {
                    match else_ret {
                        Some(r) => then_ret && r,
                        None => false,
                    }
                };
                result = all_ret;
            }
            Stmt::While {
                condition, body, ..
            } => {
                let cond_ty = self.expr_type(condition)?;
                if cond_ty != TypeKind::Bool {
                    return Err(Diagnostic::error(
                        format!(
                            "condition in `while` must be `Bool`, found `{}`",
                            cond_ty.rust_type_name()
                        ),
                        condition.span,
                    ));
                }
                self.push_scope();
                self.analyze_block(body)?;
                self.pop_scope();
                result = false;
            }
            Stmt::Return { value, span } => {
                if let Some(expr) = value {
                    let value_ty = self.expr_type(expr)?;
                    if value_ty != self.current_return_type {
                        return Err(Diagnostic::error(
                            format!(
                                "type mismatch in return: expected `{}`, found `{}`",
                                self.current_return_type.rust_type_name(),
                                value_ty.rust_type_name()
                            ),
                            expr.span,
                        ));
                    }
                } else if self.current_return_type != TypeKind::Unit {
                    return Err(Diagnostic::error(
                        format!(
                            "missing return value: expected `{}`",
                            self.current_return_type.rust_type_name()
                        ),
                        *span,
                    ));
                }
                result = true;
            }
            Stmt::Puts { value, .. } => {
                self.expr_type(value)?;
                result = false;
            }
            Stmt::Expr(expr) => {
                self.expr_type(expr)?;
                result = false;
            }
        }
        Ok(result)
    }

    fn expr_type(&mut self, expr: &Expr) -> Result<TypeKind, Diagnostic> {
        match &expr.kind {
            ExprKind::IntLit(_) => Ok(TypeKind::Int),
            ExprKind::BoolLit(_) => Ok(TypeKind::Bool),
            ExprKind::StrLit(_) => Ok(TypeKind::String),
            ExprKind::Variable(name) => {
                let var = self.lookup_var(name).ok_or_else(|| {
                    Diagnostic::error(format!("undefined variable `{name}`"), expr.span)
                })?;

                if var.moved {
                    return Err(Diagnostic::error(
                        format!("use of moved variable `{name}`"),
                        expr.span,
                    )
                    .with_hint("this variable was moved when passed by `take` in an earlier call"));
                }
                Ok(var.ty)
            }
            ExprKind::Unary { op, expr: inner } => {
                let inner_ty = self.expr_type(inner)?;
                match op {
                    UnaryOp::Neg => {
                        if inner_ty != TypeKind::Int {
                            return Err(Diagnostic::error(
                                format!(
                                    "unary `-` requires `Int`, found `{}`",
                                    inner_ty.rust_type_name()
                                ),
                                expr.span,
                            ));
                        }
                        Ok(TypeKind::Int)
                    }
                    UnaryOp::Not => {
                        if inner_ty != TypeKind::Bool {
                            return Err(Diagnostic::error(
                                format!(
                                    "unary `!` requires `Bool`, found `{}`",
                                    inner_ty.rust_type_name()
                                ),
                                expr.span,
                            ));
                        }
                        Ok(TypeKind::Bool)
                    }
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let lhs_ty = self.expr_type(lhs)?;
                let rhs_ty = self.expr_type(rhs)?;
                match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Rem => {
                        if lhs_ty != TypeKind::Int || rhs_ty != TypeKind::Int {
                            return Err(Diagnostic::error(
                                format!(
                                    "operator `{}` requires `Int` operands, found `{}` and `{}`",
                                    op.as_str(),
                                    lhs_ty.rust_type_name(),
                                    rhs_ty.rust_type_name()
                                ),
                                expr.span,
                            ));
                        }
                        Ok(TypeKind::Int)
                    }
                    BinaryOp::Eq | BinaryOp::NotEq => {
                        if lhs_ty != rhs_ty {
                            return Err(Diagnostic::error(
                                format!(
                                    "operator `{}` requires operands of the same type, found `{}` and `{}`",
                                    op.as_str(),
                                    lhs_ty.rust_type_name(),
                                    rhs_ty.rust_type_name()
                                ),
                                expr.span,
                            ));
                        }
                        Ok(TypeKind::Bool)
                    }
                    BinaryOp::Less
                    | BinaryOp::Greater
                    | BinaryOp::LessEq
                    | BinaryOp::GreaterEq => {
                        if lhs_ty != rhs_ty {
                            return Err(Diagnostic::error(
                                format!(
                                    "comparison `{}` requires operands of the same type, found `{}` and `{}`",
                                    op.as_str(),
                                    lhs_ty.rust_type_name(),
                                    rhs_ty.rust_type_name()
                                ),
                                expr.span,
                            ));
                        }
                        if lhs_ty != TypeKind::Int
                            && lhs_ty != TypeKind::Bool
                            && lhs_ty != TypeKind::String
                        {
                            return Err(Diagnostic::error(
                                format!(
                                    "comparison requires `Int`, `Bool` or `String` operands, found `{}`",
                                    lhs_ty.rust_type_name()
                                ),
                                expr.span,
                            ));
                        }
                        Ok(TypeKind::Bool)
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if lhs_ty != TypeKind::Bool || rhs_ty != TypeKind::Bool {
                            return Err(Diagnostic::error(
                                format!(
                                    "operator `{}` requires `Bool` operands, found `{}` and `{}`",
                                    op.as_str(),
                                    lhs_ty.rust_type_name(),
                                    rhs_ty.rust_type_name()
                                ),
                                expr.span,
                            ));
                        }
                        Ok(TypeKind::Bool)
                    }
                }
            }
            ExprKind::Call { callee, args } => {
                let (params_sig, return_ty, _) = self
                    .function_signatures
                    .get(callee)
                    .cloned()
                    .ok_or_else(|| {
                        Diagnostic::error(
                            format!("call to undefined function `{callee}`"),
                            expr.span,
                        )
                    })?;

                if args.len() != params_sig.len() {
                    return Err(Diagnostic::error(
                        format!(
                            "function `{callee}` expects {} argument(s), but {} were provided",
                            params_sig.len(),
                            args.len()
                        ),
                        expr.span,
                    ));
                }

                for (i, arg) in args.iter().enumerate() {
                    let (mode, param_ty) = params_sig[i];
                    match mode {
                        ParamMode::Mut => {
                            let var_name = match &arg.kind {
                                ExprKind::Variable(name) => name,
                                _ => {
                                    return Err(Diagnostic::error(
                                        "argument to `mut` parameter must be a variable",
                                        arg.span,
                                    ));
                                }
                            };

                            let var = self.lookup_var(var_name).ok_or_else(|| {
                                Diagnostic::error(
                                    format!("undefined variable `{var_name}`"),
                                    arg.span,
                                )
                            })?;

                            if var.moved {
                                return Err(Diagnostic::error(
                                    format!("use of moved variable `{var_name}`"),
                                    arg.span,
                                )
                                .with_hint("this variable was moved in an earlier call"));
                            }

                            if !var.mutable {
                                return Err(Diagnostic::error(
                                    format!(
                                        "cannot pass immutable variable `{var_name}` as mutable argument"
                                    ),
                                    arg.span,
                                )
                                .with_hint("the parameter requires `mut` access; consider declaring with `let mut`"));
                            }

                            if var.ty != param_ty {
                                return Err(Diagnostic::error(
                                    format!(
                                        "function `{callee}` expects argument {} to be `{}`, found `{}`",
                                        i + 1,
                                        param_ty.rust_type_name(),
                                        var.ty.rust_type_name()
                                    ),
                                    arg.span,
                                ));
                            }
                        }
                        ParamMode::Take => {
                            let arg_ty = self.expr_type(arg)?;
                            if arg_ty != param_ty {
                                return Err(Diagnostic::error(
                                    format!(
                                        "function `{callee}` expects argument {} to be `{}`, found `{}`",
                                        i + 1,
                                        param_ty.rust_type_name(),
                                        arg_ty.rust_type_name()
                                    ),
                                    arg.span,
                                ));
                            }
                            if param_ty == TypeKind::String {
                                if let ExprKind::Variable(var_name) = &arg.kind {
                                    if let Some(var) = self.lookup_var_mut(var_name) {
                                        if var.ty == TypeKind::String {
                                            var.moved = true;
                                        }
                                    }
                                }
                            }
                        }
                        ParamMode::Read => {
                            let arg_ty = self.expr_type(arg)?;
                            if arg_ty != param_ty {
                                return Err(Diagnostic::error(
                                    format!(
                                        "function `{callee}` expects argument {} to be `{}`, found `{}`",
                                        i + 1,
                                        param_ty.rust_type_name(),
                                        arg_ty.rust_type_name()
                                    ),
                                    arg.span,
                                ));
                            }
                        }
                    }
                }
                Ok(return_ty)
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

    fn lookup_var_mut(&mut self, name: &str) -> Option<&mut VarInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.variables.get_mut(name) {
                return Some(info);
            }
        }
        None
    }
}

trait TypeKindExt {
    fn rust_type_name(&self) -> &'static str;
}

impl TypeKindExt for TypeKind {
    fn rust_type_name(&self) -> &'static str {
        match self {
            TypeKind::Int => "Int",
            TypeKind::Bool => "Bool",
            TypeKind::String => "String",
            TypeKind::Unit => "Unit",
        }
    }
}
