use crate::ast::*;

pub struct CodeGenerator;

impl CodeGenerator {
    pub fn generate(program: &Program) -> String {
        let mut out = String::new();
        out.push_str("#![forbid(unsafe_code)]\n");
        out.push_str("#![allow(unused_parens)]\n\n");

        for func in &program.functions {
            Self::generate_function(func, &mut out);
            out.push('\n');
        }

        out
    }

    fn generate_function(func: &FunctionDecl, out: &mut String) {
        let params: Vec<String> = func
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.ty.kind.rust_type()))
            .collect();

        out.push_str(&format!(
            "fn {}({}) -> {} {{\n",
            func.name,
            params.join(", "),
            func.ret_ty.kind.rust_type()
        ));

        for stmt in &func.body {
            Self::generate_stmt(stmt, out, 1);
        }

        out.push_str("}\n");
    }

    fn generate_stmt(stmt: &Stmt, out: &mut String, indent_level: usize) {
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
                    Self::generate_expr(value)
                ));
            }
            Stmt::Assign { target, value, .. } => {
                out.push_str(&format!(
                    "{indent}{target} = {};\n",
                    Self::generate_expr(value)
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
                    Self::generate_expr(condition)
                ));
                for s in then_branch {
                    Self::generate_stmt(s, out, indent_level + 1);
                }
                if let Some(else_stmts) = else_branch {
                    out.push_str(&format!("{indent}}} else {{\n"));
                    for s in else_stmts {
                        Self::generate_stmt(s, out, indent_level + 1);
                    }
                }
                out.push_str(&format!("{indent}}}\n"));
            }
            Stmt::While {
                condition, body, ..
            } => {
                out.push_str(&format!(
                    "{indent}while {} {{\n",
                    Self::generate_expr(condition)
                ));
                for s in body {
                    Self::generate_stmt(s, out, indent_level + 1);
                }
                out.push_str(&format!("{indent}}}\n"));
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    out.push_str(&format!("{indent}return {};\n", Self::generate_expr(expr)));
                } else {
                    out.push_str(&format!("{indent}return;\n"));
                }
            }
            Stmt::Puts { value, .. } => {
                out.push_str(&format!(
                    "{indent}println!(\"{{}}\", {});\n",
                    Self::generate_expr(value)
                ));
            }
            Stmt::Expr(expr) => {
                out.push_str(&format!("{indent}{};\n", Self::generate_expr(expr)));
            }
        }
    }

    fn generate_expr(expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::IntLit(val) => format!("{val}i64"),
            ExprKind::BoolLit(val) => format!("{val}"),
            ExprKind::StrLit(val) => format!("{val:?}.to_string()"),
            ExprKind::Variable(name) => name.clone(),
            ExprKind::Unary { op, expr } => {
                format!("({}{})", op.as_str(), Self::generate_expr(expr))
            }
            ExprKind::Binary { op, lhs, rhs } => {
                format!(
                    "({} {} {})",
                    Self::generate_expr(lhs),
                    op.as_str(),
                    Self::generate_expr(rhs)
                )
            }
            ExprKind::Call { callee, args } => {
                let gen_args: Vec<String> = args.iter().map(Self::generate_expr).collect();
                format!("{callee}({})", gen_args.join(", "))
            }
        }
    }
}
