use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Def,
    End,
    If,
    Else,
    While,
    Let,
    Mut,
    Read,
    Take,
    Return,
    Puts,
    True,
    False,

    // Types
    TyInt,
    TyBool,
    TyString,
    TyUnit,

    // Literals and Identifiers
    Ident(String),
    Number(String),
    Str(String),

    // Symbols
    LParen,
    RParen,
    Comma,
    Colon,
    Arrow,
    Equal,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    NotEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    AndAnd,
    OrOr,
    Bang,

    // Layout
    Newline,
    Eof,
}

impl TokenKind {
    pub fn is_reserved_keyword(s: &str) -> bool {
        matches!(
            s,
            "fn" | "let" | "mut" | "read" | "take" | "if" | "else" | "while" | "loop" | "match" | "use" | "mod"
                | "pub" | "unsafe" | "as" | "return" | "true" | "false" | "end" | "def" | "do"
                | "struct" | "impl" | "trait" | "enum" | "const" | "static" | "move" | "ref"
                | "self" | "Self" | "crate" | "super" | "type" | "where" | "async" | "await"
                | "extern" | "puts"
        )
    }

    pub fn description(&self) -> &'static str {
        match self {
            TokenKind::Def => "`def`",
            TokenKind::End => "`end`",
            TokenKind::If => "`if`",
            TokenKind::Else => "`else`",
            TokenKind::While => "`while`",
            TokenKind::Let => "`let`",
            TokenKind::Mut => "`mut`",
            TokenKind::Read => "`read`",
            TokenKind::Take => "`take`",
            TokenKind::Return => "`return`",
            TokenKind::Puts => "`puts`",
            TokenKind::True => "`true`",
            TokenKind::False => "`false`",
            TokenKind::TyInt => "`Int`",
            TokenKind::TyBool => "`Bool`",
            TokenKind::TyString => "`String`",
            TokenKind::TyUnit => "`Unit`",
            TokenKind::Ident(_) => "identifier",
            TokenKind::Number(_) => "number",
            TokenKind::Str(_) => "string",
            TokenKind::LParen => "`(`",
            TokenKind::RParen => "`)`",
            TokenKind::Comma => "`,`",
            TokenKind::Colon => "`:`",
            TokenKind::Arrow => "`->`",
            TokenKind::Equal => "`=`",
            TokenKind::Plus => "`+`",
            TokenKind::Minus => "`-`",
            TokenKind::Star => "`*`",
            TokenKind::Slash => "`/`",
            TokenKind::Percent => "`%`",
            TokenKind::EqEq => "`==`",
            TokenKind::NotEq => "`!=`",
            TokenKind::Less => "`<`",
            TokenKind::Greater => "`>`",
            TokenKind::LessEq => "`<=`",
            TokenKind::GreaterEq => "`>=`",
            TokenKind::AndAnd => "`&&`",
            TokenKind::OrOr => "`||`",
            TokenKind::Bang => "`!`",
            TokenKind::Newline => "newline",
            TokenKind::Eof => "end of input",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
