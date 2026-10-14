use crate::ast::*;
use crate::diagnostics::Diagnostic;
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(tokens: Vec<Token>) -> Result<Program, Diagnostic> {
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }

    fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut functions = Vec::new();
        self.skip_newlines();

        while !self.is_eof() {
            functions.push(self.parse_function()?);
            self.skip_newlines();
        }

        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<FunctionDecl, Diagnostic> {
        let def_token = self.expect(TokenKind::Def)?;
        let start_span = def_token.span;

        let name_token = self.expect_ident()?;
        let func_name = match name_token.kind {
            TokenKind::Ident(s) => s,
            _ => unreachable!(),
        };

        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();

        if !self.check(&TokenKind::RParen) {
            loop {
                let p_name_tok = self.expect_ident()?;
                let p_name = match p_name_tok.kind {
                    TokenKind::Ident(s) => s,
                    _ => unreachable!(),
                };

                self.expect(TokenKind::Colon)?;
                let p_ty = self.parse_type()?;
                let p_span = p_name_tok.span.combine(p_ty.span);

                params.push(Param {
                    name: p_name,
                    ty: p_ty,
                    span: p_span,
                });

                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let rparen_token = self.expect(TokenKind::RParen)?;

        let ret_ty = if self.check(&TokenKind::Arrow) {
            self.advance();
            self.parse_type()?
        } else {
            Type {
                kind: TypeKind::Unit,
                span: rparen_token.span,
            }
        };

        self.expect_statement_separator()?;

        let mut body = Vec::new();
        self.skip_newlines();

        while !self.check(&TokenKind::End) && !self.is_eof() {
            body.push(self.parse_stmt()?);
            self.skip_newlines();
        }

        let end_token = self.expect(TokenKind::End)?;
        let func_span = start_span.combine(end_token.span);

        Ok(FunctionDecl {
            name: func_name,
            params,
            ret_ty,
            body,
            span: func_span,
        })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        match self.peek_kind() {
            TokenKind::Let => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Puts => self.parse_puts(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Ident(_) if self.peek_next_kind() == Some(&TokenKind::Equal) => {
                self.parse_assignment()
            }
            _ => {
                let expr = self.parse_expr()?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, Diagnostic> {
        let let_token = self.expect(TokenKind::Let)?;
        let mutable = if self.check(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        let name_token = self.expect_ident()?;
        let var_name = match name_token.kind {
            TokenKind::Ident(s) => s,
            _ => unreachable!(),
        };

        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Equal)?;
        let value = self.parse_expr()?;
        let span = let_token.span.combine(value.span);

        Ok(Stmt::Let {
            mutable,
            name: var_name,
            ty,
            value,
            span,
        })
    }

    fn parse_assignment(&mut self) -> Result<Stmt, Diagnostic> {
        let name_token = self.expect_ident()?;
        let target = match name_token.kind {
            TokenKind::Ident(s) => s,
            _ => unreachable!(),
        };
        self.expect(TokenKind::Equal)?;
        let value = self.parse_expr()?;
        let span = name_token.span.combine(value.span);

        Ok(Stmt::Assign {
            target,
            value,
            span,
        })
    }

    fn parse_puts(&mut self) -> Result<Stmt, Diagnostic> {
        let puts_tok = self.expect(TokenKind::Puts)?;
        let value = self.parse_expr()?;
        let span = puts_tok.span.combine(value.span);
        Ok(Stmt::Puts { value, span })
    }

    fn parse_return(&mut self) -> Result<Stmt, Diagnostic> {
        let ret_tok = self.expect(TokenKind::Return)?;
        if self.check(&TokenKind::Newline) || self.check(&TokenKind::End) || self.is_eof() {
            Ok(Stmt::Return {
                value: None,
                span: ret_tok.span,
            })
        } else {
            let expr = self.parse_expr()?;
            let span = ret_tok.span.combine(expr.span);
            Ok(Stmt::Return {
                value: Some(expr),
                span,
            })
        }
    }

    fn parse_if(&mut self) -> Result<Stmt, Diagnostic> {
        let if_tok = self.expect(TokenKind::If)?;
        let condition = self.parse_expr()?;
        self.expect_statement_separator()?;

        let mut then_branch = Vec::new();
        self.skip_newlines();

        while !self.check(&TokenKind::Else) && !self.check(&TokenKind::End) && !self.is_eof() {
            then_branch.push(self.parse_stmt()?);
            self.skip_newlines();
        }

        let else_branch = if self.check(&TokenKind::Else) {
            self.advance();
            self.expect_statement_separator()?;
            let mut else_stmts = Vec::new();
            self.skip_newlines();
            while !self.check(&TokenKind::End) && !self.is_eof() {
                else_stmts.push(self.parse_stmt()?);
                self.skip_newlines();
            }
            Some(else_stmts)
        } else {
            None
        };

        let end_tok = self.expect(TokenKind::End)?;
        let span = if_tok.span.combine(end_tok.span);

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, Diagnostic> {
        let while_tok = self.expect(TokenKind::While)?;
        let condition = self.parse_expr()?;
        self.expect_statement_separator()?;

        let mut body = Vec::new();
        self.skip_newlines();

        while !self.check(&TokenKind::End) && !self.is_eof() {
            body.push(self.parse_stmt()?);
            self.skip_newlines();
        }

        let end_tok = self.expect(TokenKind::End)?;
        let span = while_tok.span.combine(end_tok.span);

        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }

    fn parse_type(&mut self) -> Result<Type, Diagnostic> {
        let token = self.advance();
        let kind = match token.kind {
            TokenKind::TyInt => TypeKind::Int,
            TokenKind::TyBool => TypeKind::Bool,
            TokenKind::TyString => TypeKind::String,
            TokenKind::TyUnit => TypeKind::Unit,
            _ => {
                return Err(Diagnostic::error(
                    format!(
                        "expected type (`Int`, `Bool`, `String`, `Unit`), found {}",
                        token.kind.description()
                    ),
                    token.span,
                ));
            }
        };

        Ok(Type {
            kind,
            span: token.span,
        })
    }

    pub fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        self.parse_binary(0)
    }

    fn parse_binary(&mut self, min_prec: u8) -> Result<Expr, Diagnostic> {
        let mut lhs = self.parse_unary()?;

        loop {
            let Some((op, prec)) = self.peek_binary_op() else {
                break;
            };

            if prec < min_prec {
                break;
            }

            self.advance(); // consume operator
            let rhs = self.parse_binary(prec + 1)?;
            let span = lhs.span.combine(rhs.span);

            lhs = Expr::new(
                ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }

        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, Diagnostic> {
        if self.check(&TokenKind::Minus) {
            let op_tok = self.advance();
            let operand = self.parse_unary()?;
            let span = op_tok.span.combine(operand.span);
            return Ok(Expr::new(
                ExprKind::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(operand),
                },
                span,
            ));
        }

        if self.check(&TokenKind::Bang) {
            let op_tok = self.advance();
            let operand = self.parse_unary()?;
            let span = op_tok.span.combine(operand.span);
            return Ok(Expr::new(
                ExprKind::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(operand),
                },
                span,
            ));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::Number(num_str) => {
                let val = num_str.parse::<i64>().map_err(|_| {
                    Diagnostic::error(format!("integer literal out of range: `{num_str}`"), token.span)
                })?;
                Ok(Expr::new(ExprKind::IntLit(val), token.span))
            }
            TokenKind::Str(s) => Ok(Expr::new(ExprKind::StrLit(s), token.span)),
            TokenKind::True => Ok(Expr::new(ExprKind::BoolLit(true), token.span)),
            TokenKind::False => Ok(Expr::new(ExprKind::BoolLit(false), token.span)),
            TokenKind::Ident(name) => {
                if self.check(&TokenKind::LParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.check(&TokenKind::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    let rparen = self.expect(TokenKind::RParen)?;
                    let span = token.span.combine(rparen.span);
                    Ok(Expr::new(ExprKind::Call { callee: name, args }, span))
                } else {
                    Ok(Expr::new(ExprKind::Variable(name), token.span))
                }
            }
            TokenKind::LParen => {
                let inner = self.parse_expr()?;
                let rparen = self.expect(TokenKind::RParen)?;
                let span = token.span.combine(rparen.span);
                Ok(Expr::new(inner.kind, span))
            }
            other => Err(Diagnostic::error(
                format!("expected expression, found {}", other.description()),
                token.span,
            )),
        }
    }

    fn peek_binary_op(&self) -> Option<(BinaryOp, u8)> {
        match self.peek_kind() {
            TokenKind::OrOr => Some((BinaryOp::Or, 1)),
            TokenKind::AndAnd => Some((BinaryOp::And, 2)),
            TokenKind::EqEq => Some((BinaryOp::Eq, 3)),
            TokenKind::NotEq => Some((BinaryOp::NotEq, 3)),
            TokenKind::Less => Some((BinaryOp::Less, 4)),
            TokenKind::LessEq => Some((BinaryOp::LessEq, 4)),
            TokenKind::Greater => Some((BinaryOp::Greater, 4)),
            TokenKind::GreaterEq => Some((BinaryOp::GreaterEq, 4)),
            TokenKind::Plus => Some((BinaryOp::Add, 5)),
            TokenKind::Minus => Some((BinaryOp::Sub, 5)),
            TokenKind::Star => Some((BinaryOp::Mul, 6)),
            TokenKind::Slash => Some((BinaryOp::Div, 6)),
            TokenKind::Percent => Some((BinaryOp::Rem, 6)),
            _ => None,
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<Token, Diagnostic> {
        let token = self.advance();
        if token.kind == expected {
            Ok(token)
        } else {
            Err(Diagnostic::error(
                format!(
                    "expected {}, found {}",
                    expected.description(),
                    token.kind.description()
                ),
                token.span,
            ))
        }
    }

    fn expect_ident(&mut self) -> Result<Token, Diagnostic> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Ident(_)) {
            Ok(token)
        } else {
            Err(Diagnostic::error(
                format!("expected identifier, found {}", token.kind.description()),
                token.span,
            ))
        }
    }

    fn expect_statement_separator(&mut self) -> Result<(), Diagnostic> {
        if self.check(&TokenKind::Newline) {
            self.advance();
            self.skip_newlines();
            Ok(())
        } else if self.is_eof() {
            Ok(())
        } else {
            let tok = self.peek();
            Err(Diagnostic::error(
                format!(
                    "expected newline after statement, found {}",
                    tok.kind.description()
                ),
                tok.span,
            ))
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(&TokenKind::Newline) {
            self.advance();
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind() == kind
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len().saturating_sub(1))]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn peek_next_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos + 1).map(|t| &t.kind)
    }

    fn advance(&mut self) -> Token {
        if !self.is_eof() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            tok
        } else {
            self.peek().clone()
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }
}
