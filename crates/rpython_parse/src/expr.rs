use rpython_ast::TyId;
use rpython_ast::{BinaryOp, ExprId, ExprKind, FieldExpr, Literal, Path, PathSegment, UnaryOp};
use rpython_span::Span;
use rpython_syntax::{IntLiteral, TokenKind};
use smol_str::SmolStr;

use crate::parser::Parser;

impl Parser<'_> {
    pub fn parse_expr(&mut self) -> Option<ExprId> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Option<ExprId> {
        self.skip_layout();
        let start = self.current().span;
        let mut lhs = self.parse_prefix()?;

        loop {
            self.skip_layout();
            if matches!(self.current_kind(), TokenKind::Assign) {
                break;
            }

            let (l_bp, r_bp, op) = match self.infix_op() {
                Some(v) => v,
                None => break,
            };
            if l_bp < min_bp {
                break;
            }
            self.bump();
            self.skip_layout();
            let rhs = self.parse_expr_bp(r_bp)?;
            let span = self.span_from(start);
            lhs = self.arena.alloc_expr(
                ExprKind::Binary {
                    op,
                    left: lhs,
                    right: rhs,
                },
                span,
            );
        }

        Some(lhs)
    }

    fn infix_op(&self) -> Option<(u8, u8, BinaryOp)> {
        use TokenKind::*;
        let (l_bp, r_bp, op) = match self.current_kind() {
            KwOr => (1, 2, BinaryOp::Or),
            KwAnd => (3, 4, BinaryOp::And),
            EqEq => (5, 6, BinaryOp::Eq),
            NotEq => (5, 6, BinaryOp::NotEq),
            Lt => (5, 6, BinaryOp::Lt),
            LtEq => (5, 6, BinaryOp::LtEq),
            Gt => (5, 6, BinaryOp::Gt),
            GtEq => (5, 6, BinaryOp::GtEq),
            Plus => (7, 8, BinaryOp::Add),
            Minus => (7, 8, BinaryOp::Sub),
            Star => (9, 10, BinaryOp::Mul),
            Slash => (9, 10, BinaryOp::Div),
            FloorDiv => (9, 10, BinaryOp::FloorDiv),
            Percent => (9, 10, BinaryOp::Mod),
            _ => return None,
        };
        Some((l_bp, r_bp, op))
    }

    fn parse_prefix(&mut self) -> Option<ExprId> {
        let start = self.current().span;
        let kind = match self.current_kind().clone() {
            TokenKind::Bang => {
                self.bump();
                self.skip_layout();
                let operand = self.parse_expr_bp(11)?;
                ExprKind::Unary {
                    op: UnaryOp::Not,
                    operand,
                }
            }
            TokenKind::Minus => {
                self.bump();
                self.skip_layout();
                let operand = self.parse_expr_bp(11)?;
                ExprKind::Unary {
                    op: UnaryOp::Neg,
                    operand,
                }
            }
            TokenKind::Plus => {
                self.bump();
                self.skip_layout();
                let operand = self.parse_expr_bp(11)?;
                ExprKind::Unary {
                    op: UnaryOp::Pos,
                    operand,
                }
            }
            _ => {
                let expr = self.parse_atom()?;
                return self.parse_postfix(expr);
            }
        };
        let span = self.span_from(start);
        Some(self.arena.alloc_expr(kind, span))
    }

    fn parse_atom(&mut self) -> Option<ExprId> {
        let start = self.current().span;
        let kind = match self.current_kind().clone() {
            TokenKind::IntLit { value } => {
                self.bump();
                ExprKind::Literal(self.int_literal(value))
            }
            TokenKind::FloatLit { value } => {
                self.bump();
                ExprKind::Literal(Literal::Float(value))
            }
            TokenKind::StringLit { value } => {
                self.bump();
                ExprKind::Literal(Literal::String(SmolStr::new(value)))
            }
            TokenKind::BytesLit { value } => {
                self.bump();
                ExprKind::Literal(Literal::Bytes(value))
            }
            TokenKind::BoolLit(value) => {
                self.bump();
                ExprKind::Literal(Literal::Bool(value))
            }
            TokenKind::KwTrue => {
                self.bump();
                ExprKind::Literal(Literal::Bool(true))
            }
            TokenKind::KwFalse => {
                self.bump();
                ExprKind::Literal(Literal::Bool(false))
            }
            TokenKind::KwNone => {
                self.bump();
                ExprKind::Literal(Literal::None)
            }
            TokenKind::KwSelf => {
                self.bump();
                ExprKind::Path(Path::single(SmolStr::new("self"), start))
            }
            TokenKind::Ident { name } => {
                self.bump();
                if name == "True" {
                    ExprKind::Literal(Literal::Bool(true))
                } else if name == "False" {
                    ExprKind::Literal(Literal::Bool(false))
                } else {
                    ExprKind::Path(Path::single(name, start))
                }
            }
            TokenKind::LParen => {
                self.bump();
                self.skip_layout();
                if self.eat(TokenKind::RParen) {
                    ExprKind::Tuple(Vec::new())
                } else {
                    let first = self.parse_expr()?;
                    self.skip_layout();
                    if self.eat(TokenKind::Comma) {
                        let mut elems = vec![first];
                        loop {
                            self.skip_layout();
                            if self.eat(TokenKind::RParen) {
                                break;
                            }
                            elems.push(self.parse_expr()?);
                            self.skip_layout();
                            if !self.eat(TokenKind::Comma) {
                                if !self.expect(TokenKind::RParen, "expected ')'") {
                                    return None;
                                }
                                break;
                            }
                        }
                        ExprKind::Tuple(elems)
                    } else {
                        if !self.expect(TokenKind::RParen, "expected ')'") {
                            return None;
                        }
                        return Some(first);
                    }
                }
            }
            TokenKind::LBracket => {
                self.bump();
                self.skip_layout();
                let mut elems = Vec::new();
                if !self.eat(TokenKind::RBracket) {
                    loop {
                        elems.push(self.parse_expr()?);
                        self.skip_layout();
                        if self.eat(TokenKind::RBracket) {
                            break;
                        }
                        if !self.eat(TokenKind::Comma) {
                            self.error(self.current().span, "expected ',' or ']'");
                            return None;
                        }
                        self.skip_layout();
                    }
                }
                ExprKind::List(elems)
            }
            TokenKind::LBrace => {
                self.bump();
                let path = Path {
                    segments: Vec::new(),
                    span: start,
                };
                let fields = self.parse_struct_fields(start)?;
                let span = self.span_from(start);
                return Some(
                    self.arena
                        .alloc_expr(ExprKind::Struct { path, fields }, span),
                );
            }
            TokenKind::KwIf => {
                return self.parse_if_expr(start);
            }
            _ => {
                self.error(
                    start,
                    format!("expected expression, found {}", self.current().kind.name()),
                );
                return None;
            }
        };
        let span = self.span_from(start);
        Some(self.arena.alloc_expr(kind, span))
    }

    fn int_literal(&self, lit: IntLiteral) -> Literal {
        let value = match lit {
            IntLiteral::Decimal(v) | IntLiteral::Hex(v) | IntLiteral::Bin(v) => v,
        };
        Literal::Int(value)
    }

    fn parse_if_expr(&mut self, start: Span) -> Option<ExprId> {
        self.bump();
        self.skip_layout();
        let test = self.parse_expr()?;
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' after if condition");
            return None;
        }
        self.skip_layout();
        let then_branch = self.parse_expr()?;
        self.skip_layout();
        if !self.eat(TokenKind::KwElse) {
            self.error(self.current().span, "expected 'else' in if expression");
            return None;
        }
        self.skip_layout();
        let else_branch = self.parse_expr()?;
        let span = Span::merge(start, self.arena.expr(else_branch).span);
        Some(self.arena.alloc_expr(
            ExprKind::If {
                test,
                then: then_branch,
                else_branch,
            },
            span,
        ))
    }

    fn parse_struct_fields(&mut self, field_span: Span) -> Option<Vec<FieldExpr>> {
        self.skip_layout();
        let mut fields = Vec::new();
        if self.eat(TokenKind::RBrace) {
            return Some(fields);
        }
        loop {
            let name = match self.current_kind() {
                TokenKind::Ident { name } => name.clone(),
                _ => {
                    self.error(self.current().span, "expected field name");
                    return None;
                }
            };
            self.bump();
            if !self.eat(TokenKind::Colon) {
                self.error(self.current().span, "expected ':' in struct literal");
                return None;
            }
            self.skip_layout();
            let expr = self.parse_expr()?;
            fields.push(FieldExpr {
                name,
                expr,
                span: field_span,
            });
            self.skip_layout();
            if self.eat(TokenKind::RBrace) {
                break;
            }
            if !self.eat(TokenKind::Comma) {
                self.error(self.current().span, "expected ',' or '}'");
                return None;
            }
            self.skip_layout();
        }
        Some(fields)
    }

    fn parse_postfix(&mut self, mut expr: ExprId) -> Option<ExprId> {
        loop {
            self.skip_layout();
            match self.current_kind() {
                TokenKind::LParen => {
                    self.bump();
                    let args = self.parse_call_args()?;
                    let span =
                        Span::merge(self.arena.expr(expr).span, self.tokens[self.pos - 1].span);
                    expr = self.arena.alloc_expr(
                        ExprKind::Call {
                            func: expr,
                            args,
                            kwargs: Vec::new(),
                        },
                        span,
                    );
                }
                TokenKind::LBrace => {
                    if let ExprKind::Path(path) = &self.arena.expr(expr).kind {
                        let path = path.clone();
                        self.bump();
                        let fields = self.parse_struct_fields(self.current().span)?;
                        let span =
                            Span::merge(self.arena.expr(expr).span, self.tokens[self.pos - 1].span);
                        expr = self
                            .arena
                            .alloc_expr(ExprKind::Struct { path, fields }, span);
                    } else {
                        self.error(self.current().span, "unexpected '{'");
                        return None;
                    }
                }
                TokenKind::Dot => {
                    self.bump();
                    let field = match self.current_kind() {
                        TokenKind::Ident { name } => {
                            let name = name.clone();
                            self.bump();
                            name
                        }
                        _ => {
                            self.error(self.current().span, "expected field name after '.'");
                            return None;
                        }
                    };
                    if self.eat(TokenKind::LParen) {
                        let args = self.parse_call_args()?;
                        let span =
                            Span::merge(self.arena.expr(expr).span, self.tokens[self.pos - 1].span);
                        expr = self.arena.alloc_expr(
                            ExprKind::MethodCall {
                                receiver: expr,
                                method: field,
                                args,
                            },
                            span,
                        );
                    } else {
                        let span =
                            Span::merge(self.arena.expr(expr).span, self.tokens[self.pos - 1].span);
                        expr = self
                            .arena
                            .alloc_expr(ExprKind::Field { base: expr, field }, span);
                    }
                }
                TokenKind::LBracket => {
                    self.bump();
                    self.skip_layout();
                    let index = self.parse_expr()?;
                    self.skip_layout();
                    if !self.expect(TokenKind::RBracket, "expected ']'") {
                        return None;
                    }
                    let span =
                        Span::merge(self.arena.expr(expr).span, self.tokens[self.pos - 1].span);
                    expr = self
                        .arena
                        .alloc_expr(ExprKind::Index { base: expr, index }, span);
                }
                _ => break,
            }
        }
        Some(expr)
    }

    fn parse_call_args(&mut self) -> Option<Vec<ExprId>> {
        self.skip_layout();
        let mut args = Vec::new();
        if !self.eat(TokenKind::RParen) {
            loop {
                args.push(self.parse_expr()?);
                self.skip_layout();
                if self.eat(TokenKind::RParen) {
                    break;
                }
                if !self.eat(TokenKind::Comma) {
                    self.error(self.current().span, "expected ',' or ')'");
                    return None;
                }
            }
        }
        Some(args)
    }

    pub(crate) fn parse_path(&mut self) -> Option<Path> {
        let start = self.current().span;
        let mut segments = Vec::new();
        loop {
            let ident = match self.current_kind() {
                TokenKind::Ident { name } => name.clone(),
                TokenKind::KwSelf => {
                    self.bump();
                    SmolStr::new("self")
                }
                _ => {
                    if segments.is_empty() {
                        self.error(start, "expected path");
                        return None;
                    }
                    break;
                }
            };
            if !matches!(self.current_kind(), TokenKind::KwSelf) {
                self.bump();
            }
            let args = if self.eat(TokenKind::LBracket) {
                self.parse_type_args()?
            } else {
                Vec::new()
            };
            let seg_span = self.tokens[self.pos.saturating_sub(1)].span;
            segments.push(PathSegment {
                ident,
                args,
                span: seg_span,
            });
            if !self.eat(TokenKind::Dot) {
                break;
            }
        }
        let span = self.span_from(start);
        Some(Path { segments, span })
    }

    fn parse_type_args(&mut self) -> Option<Vec<TyId>> {
        self.skip_layout();
        let mut args = Vec::new();
        if !self.eat(TokenKind::RBracket) {
            loop {
                args.push(self.parse_ty()?);
                self.skip_layout();
                if self.eat(TokenKind::RBracket) {
                    break;
                }
                if !self.eat(TokenKind::Comma) {
                    self.error(self.current().span, "expected ',' or ']'");
                    return None;
                }
            }
        }
        Some(args)
    }
}
