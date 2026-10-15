use crate::diagnostics::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(source: &'a str) -> Result<Vec<Token>, Diagnostic> {
        let mut lexer = Lexer::new(source);
        lexer.scan_all()
    }

    fn scan_all(&mut self) -> Result<Vec<Token>, Diagnostic> {
        let mut tokens = Vec::new();
        let mut last_was_newline = true; // suppress leading newlines

        while let Some(&(start, c)) = self.chars.peek() {
            let start_line = self.line;
            let start_col = self.col;

            if c == '\n' {
                self.next_char();
                if !last_was_newline {
                    let span = Span::new(start, start + 1, start_line, start_col);
                    tokens.push(Token::new(TokenKind::Newline, span));
                    last_was_newline = true;
                }
                continue;
            }

            if c.is_whitespace() {
                self.next_char();
                continue;
            }

            if c == '#' {
                // Comment until end of line
                while let Some(&(_, ch)) = self.chars.peek() {
                    if ch == '\n' {
                        break;
                    }
                    self.next_char();
                }
                continue;
            }

            let token = self.scan_token(start, c, start_line, start_col)?;
            last_was_newline = matches!(token.kind, TokenKind::Newline);
            tokens.push(token);
        }

        // Emit EOF
        let end_pos = self.source.len();
        let eof_span = Span::new(end_pos, end_pos, self.line, self.col);
        tokens.push(Token::new(TokenKind::Eof, eof_span));

        Ok(tokens)
    }

    fn scan_token(
        &mut self,
        start: usize,
        c: char,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, Diagnostic> {
        self.next_char();

        if c == '"' {
            return self.scan_string(start, start_line, start_col);
        }

        if c.is_ascii_alphabetic() || c == '_' {
            return self.scan_ident_or_keyword(start, c, start_line, start_col);
        }

        if c.is_ascii_digit() {
            return self.scan_number(start, c, start_line, start_col);
        }

        // Symbols
        let kind = match c {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '+' => TokenKind::Plus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '-' => {
                if let Some(&(_, '>')) = self.chars.peek() {
                    self.next_char();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '=' => {
                if let Some(&(_, '=')) = self.chars.peek() {
                    self.next_char();
                    TokenKind::EqEq
                } else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if let Some(&(_, '=')) = self.chars.peek() {
                    self.next_char();
                    TokenKind::NotEq
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                if let Some(&(_, '=')) = self.chars.peek() {
                    self.next_char();
                    TokenKind::LessEq
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if let Some(&(_, '=')) = self.chars.peek() {
                    self.next_char();
                    TokenKind::GreaterEq
                } else {
                    TokenKind::Greater
                }
            }
            '&' => {
                if let Some(&(_, '&')) = self.chars.peek() {
                    self.next_char();
                    TokenKind::AndAnd
                } else {
                    let span = Span::new(start, start + 1, start_line, start_col);
                    return Err(Diagnostic::error("unsupported token: `&` (expected `&&`)", span));
                }
            }
            '|' => {
                if let Some(&(_, '|')) = self.chars.peek() {
                    self.next_char();
                    TokenKind::OrOr
                } else {
                    let span = Span::new(start, start + 1, start_line, start_col);
                    return Err(Diagnostic::error("unsupported token: `|` (expected `||`)", span));
                }
            }
            _ => {
                let span = Span::new(start, start + c.len_utf8(), start_line, start_col);
                return Err(Diagnostic::error(format!("unexpected character: '{c}'"), span));
            }
        };

        let end = self.chars.peek().map(|&(pos, _)| pos).unwrap_or(self.source.len());
        let span = Span::new(start, end, start_line, start_col);
        Ok(Token::new(kind, span))
    }

    fn scan_string(
        &mut self,
        start: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, Diagnostic> {
        let mut content = String::new();
        let mut closed = false;

        while let Some(&(_, ch)) = self.chars.peek() {
            self.next_char();
            if ch == '"' {
                closed = true;
                break;
            }
            if ch == '\\' {
                let esc_col = self.col;
                let esc_line = self.line;
                let esc_char = self
                    .chars
                    .next()
                    .map(|(_, c)| {
                        self.col += 1;
                        c
                    })
                    .ok_or_else(|| {
                        let span = Span::new(start, self.source.len(), start_line, start_col);
                        Diagnostic::error("unterminated string escape", span)
                    })?;

                match esc_char {
                    'n' => content.push('\n'),
                    'r' => content.push('\r'),
                    't' => content.push('\t'),
                    '\\' => content.push('\\'),
                    '"' => content.push('"'),
                    other => {
                        let span = Span::new(self.current_pos() - 2, self.current_pos(), esc_line, esc_col);
                        return Err(Diagnostic::error(format!("unsupported string escape: \\{other}"), span));
                    }
                }
            } else if ch == '\n' {
                let span = Span::new(start, self.current_pos(), start_line, start_col);
                return Err(Diagnostic::error("unterminated string (newline in string literal)", span));
            } else if ch.is_control() {
                let span = Span::new(start, self.current_pos(), start_line, start_col);
                return Err(Diagnostic::error("control character in string literal", span));
            } else {
                content.push(ch);
            }
        }

        if !closed {
            let span = Span::new(start, self.source.len(), start_line, start_col);
            return Err(Diagnostic::error("unterminated string", span));
        }

        let end = self.current_pos();
        let span = Span::new(start, end, start_line, start_col);
        Ok(Token::new(TokenKind::Str(content), span))
    }

    fn scan_ident_or_keyword(
        &mut self,
        start: usize,
        first: char,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, Diagnostic> {
        let mut text = String::from(first);
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                text.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        let end = self.current_pos();
        let span = Span::new(start, end, start_line, start_col);

        let kind = match text.as_str() {
            "def" => TokenKind::Def,
            "end" => TokenKind::End,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "read" => TokenKind::Read,
            "take" => TokenKind::Take,
            "return" => TokenKind::Return,
            "puts" => TokenKind::Puts,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "Int" => TokenKind::TyInt,
            "Bool" => TokenKind::TyBool,
            "String" => TokenKind::TyString,
            "Unit" => TokenKind::TyUnit,
            other if TokenKind::is_reserved_keyword(other) => {
                return Err(Diagnostic::error(
                    format!("cannot use reserved keyword `{other}` as identifier"),
                    span,
                ));
            }
            _ => TokenKind::Ident(text),
        };

        Ok(Token::new(kind, span))
    }

    fn scan_number(
        &mut self,
        start: usize,
        first: char,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, Diagnostic> {
        let mut text = String::from(first);
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_ascii_digit() {
                text.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        let end = self.current_pos();
        let span = Span::new(start, end, start_line, start_col);
        Ok(Token::new(TokenKind::Number(text), span))
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        let item = self.chars.next();
        if let Some((_, c)) = item {
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        item
    }

    fn current_pos(&mut self) -> usize {
        self.chars.peek().map(|&(pos, _)| pos).unwrap_or(self.source.len())
    }
}
