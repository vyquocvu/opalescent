use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    Int,
    Bool,
    String,
    Unit,
    /// User-defined enum. The name is interned via `Box::leak` so the type
    /// stays `Copy`; the compiler is a short-lived process.
    Enum(&'static str),
}

impl TypeKind {
    pub fn rust_type(&self) -> &'static str {
        match self {
            TypeKind::Int => "i64",
            TypeKind::Bool => "bool",
            TypeKind::String => "String",
            TypeKind::Unit => "()",
            TypeKind::Enum(name) => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamMode {
    Read,
    Mut,
    Take,
}

impl ParamMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ParamMode::Read => "read",
            ParamMode::Mut => "mut",
            ParamMode::Take => "take",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub mode: ParamMode,
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl UnaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    NotEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    And,
    Or,
}

impl BinaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Rem => "%",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Less => "<",
            BinaryOp::Greater => ">",
            BinaryOp::LessEq => "<=",
            BinaryOp::GreaterEq => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    IntLit(i64),
    BoolLit(bool),
    StrLit(String),
    Variable(String),
    /// Qualified enum variant, e.g. `Color.Red`.
    Variant {
        enum_name: String,
        variant: String,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Let {
        mutable: bool,
        name: String,
        ty: Type,
        value: Expr,
        span: Span,
    },
    Assign {
        target: String,
        value: Expr,
        span: Span,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Puts {
        value: Expr,
        span: Span,
    },
    Match {
        scrutinee: Expr,
        arms: Vec<MatchArm>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },
    Expr(Expr),
}

/// A qualified variant pattern, e.g. `Color.Red`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub enum_name: String,
    pub variant: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub patterns: Vec<Pattern>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Let { span, .. }
            | Stmt::Assign { span, .. }
            | Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::Match { span, .. }
            | Stmt::Puts { span, .. } => *span,
            Stmt::Expr(e) => e.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Type,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub enums: Vec<EnumDecl>,
    pub functions: Vec<FunctionDecl>,
}
