use std::collections::HashMap;

use crate::ast::*;

pub struct CodeGenerator<'a> {
    program: &'a Program,
    fn_signatures: HashMap<String, Vec<(ParamMode, TypeKind)>>,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(program: &'a Program) -> Self {
        let mut fn_signatures = HashMap::new();
        for func in &program.functions {
            let sig = func.params.iter().map(|p| (p.mode, p.ty.kind)).collect();
            fn_signatures.insert(func.name.clone(), sig);
        }
        Self {
            program,
            fn_signatures,
        }
    }

    pub fn generate(program: &'a Program) -> String {
        let generator = CodeGenerator::new(program);
        generator.emit()
    }

    fn emit(&self) -> String {
        let mut out = String::new();
        out.push_str("#![forbid(unsafe_code)]\n");
        out.push_str("#![allow(unused_parens)]\n");
        out.push_str("#![allow(unused_variables)]\n");
        out.push_str("#![allow(dead_code)]\n\n");

        for decl in &self.program.enums {
            self.generate_enum(decl, &mut out);
            out.push('\n');
        }

        for func in &self.program.functions {
            self.generate_function(func, &mut out);
            out.push('\n');
        }

        out
    }

    fn generate_enum(&self, decl: &EnumDecl, out: &mut String) {
        let variants: Vec<&str> = decl.variants.iter().map(|v| v.name.as_str()).collect();
        out.push_str("#[derive(Clone, Copy, PartialEq, Eq, Debug)]\n");
        out.push_str(&format!(
            "enum {} {{ {} }}\n",
            decl.name,
            variants.join(", ")
        ));
    }

    fn generate_function(&self, func: &FunctionDecl, out: &mut String) {
        let params: Vec<String> = func
            .params
            .iter()
            .map(|p| {
                let ty = p.ty.kind.rust_type();
                match p.mode {
                    ParamMode::Read => {
                        if p.ty.kind == TypeKind::String {
                            format!("{}: &str", p.name)
                        } else {
                            format!("{}: {ty}", p.name)
                        }
                    }
                    ParamMode::Mut => {
                        format!("{}: &mut {ty}", p.name)
                    }
                    ParamMode::Take => {
                        format!("{}: {ty}", p.name)
                    }
                }
            })
            .collect();

        out.push_str(&format!(
            "fn {}({}) -> {} {{\n",
            func.name,
            params.join(", "),
            func.ret_ty.kind.rust_type()
        ));

        for stmt in &func.body {
            self.generate_stmt(stmt, out, 1);
        }

        out.push_str("}\n");
    }

    fn generate_stmt(&self, stmt: &Stmt, out: &mut String, indent_level: usize) {
        let indent = "    ".repeat(indent_level);
        match stmt {
            Stmt::Let {
                mutable,
                name,
                ty,
                value,
                ..
            } => {
                let mut_str = if *mutable { "mut " } else { "" };
                out.push_str(&format!(
                    "{indent}let {mut_str}{name}: {} = {};\n",
                    ty.kind.rust_type(),
                    self.generate_expr(value)
                ));
            }
            Stmt::Assign { target, value, .. } => {
                out.push_str(&format!(
                    "{indent}{target} = {};\n",
                    self.generate_expr(value)
                ));
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                out.push_str(&format!(
                    "{indent}if {} {{\n",
                    self.generate_expr(condition)
                ));
                for s in then_branch {
                    self.generate_stmt(s, out, indent_level + 1);
                }
                if let Some(else_stmts) = else_branch {
                    out.push_str(&format!("{indent}}} else {{\n"));
                    for s in else_stmts {
                        self.generate_stmt(s, out, indent_level + 1);
                    }
                }
                out.push_str(&format!("{indent}}}\n"));
            }
            Stmt::While {
                condition, body, ..
            } => {
                out.push_str(&format!(
                    "{indent}while {} {{\n",
                    self.generate_expr(condition)
                ));
                for s in body {
                    self.generate_stmt(s, out, indent_level + 1);
                }
                out.push_str(&format!("{indent}}}\n"));
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    out.push_str(&format!("{indent}return {};\n", self.generate_expr(expr)));
                } else {
                    out.push_str(&format!("{indent}return;\n"));
                }
            }
            Stmt::Puts { value, .. } => {
                out.push_str(&format!(
                    "{indent}println!(\"{{}}\", {});\n",
                    self.generate_expr(value)
                ));
            }
            Stmt::Match {
                scrutinee,
                arms,
                else_branch,
                ..
            } => {
                out.push_str(&format!(
                    "{indent}match {} {{\n",
                    self.generate_expr(scrutinee)
                ));
                for arm in arms {
                    let patterns: Vec<String> = arm
                        .patterns
                        .iter()
                        .map(|p| format!("{}::{}", p.enum_name, p.variant))
                        .collect();
                    out.push_str(&format!(
                        "{indent}    {} => {{\n",
                        patterns.join(" | ")
                    ));
                    for s in &arm.body {
                        self.generate_stmt(s, out, indent_level + 2);
                    }
                    out.push_str(&format!("{indent}    }}\n"));
                }
                if let Some(stmts) = else_branch {
                    out.push_str(&format!("{indent}    _ => {{\n"));
                    for s in stmts {
                        self.generate_stmt(s, out, indent_level + 2);
                    }
                    out.push_str(&format!("{indent}    }}\n"));
                }
                out.push_str(&format!("{indent}}}\n"));
            }
            Stmt::Expr(expr) => {
                out.push_str(&format!("{indent}{};\n", self.generate_expr(expr)));
            }
        }
    }

    fn generate_expr(&self, expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::IntLit(val) => format!("{val}i64"),
            ExprKind::BoolLit(val) => format!("{val}"),
            ExprKind::StrLit(val) => format!("{val:?}.to_string()"),
            ExprKind::Variable(name) => name.clone(),
            ExprKind::Variant { enum_name, variant } => format!("{enum_name}::{variant}"),
            ExprKind::Unary { op, expr } => {
                format!("({}{})", op.as_str(), self.generate_expr(expr))
            }
            ExprKind::Binary { op, lhs, rhs } => {
                format!(
                    "({} {} {})",
                    self.generate_expr(lhs),
                    op.as_str(),
                    self.generate_expr(rhs)
                )
            }
            ExprKind::Call { callee, args } => {
                let sig = self.fn_signatures.get(callee);
                let gen_args: Vec<String> = args
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| {
                        if let Some(params) = sig {
                            if let Some(&(mode, ty)) = params.get(i) {
                                match mode {
                                    ParamMode::Mut => {
                                        let inner = self.generate_expr(arg);
                                        return format!("&mut {inner}");
                                    }
                                    ParamMode::Read => {
                                        if ty == TypeKind::String {
                                            if let ExprKind::StrLit(s) = &arg.kind {
                                                return format!("{s:?}");
                                            }
                                            let inner = self.generate_expr(arg);
                                            return format!("&{inner}");
                                        }
                                    }
                                    ParamMode::Take => {}
                                }
                            }
                        }
                        self.generate_expr(arg)
                    })
                    .collect();
                format!("{callee}({})", gen_args.join(", "))
            }
        }
    }
}
