use rpython_ast::{
    ElifArm, MatchArm, PatKind, PatMutability, StmtKind,
};
use rpython_ast::{ExprId, PatId, StmtId};
use rpython_span::Span;
use rpython_syntax::TokenKind;

use crate::parser::Parser;

impl Parser<'_> {
    pub fn parse_stmt(&mut self) -> Option<StmtId> {
        self.skip_stmt_separators();
        if matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
            return None;
        }

        let start = self.current().span;
        let kind = if self.eat(TokenKind::KwPass) {
            StmtKind::Pass
        } else if self.eat(TokenKind::KwBreak) {
            StmtKind::Break(None)
        } else if self.eat(TokenKind::KwContinue) {
            StmtKind::Continue(None)
        } else if self.eat(TokenKind::KwReturn) {
            self.skip_layout();
            let value = if matches!(
                self.current_kind(),
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof
            ) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            StmtKind::Return(value)
        } else if self.eat(TokenKind::KwIf) {
            return self.parse_if_stmt(start);
        } else if self.eat(TokenKind::KwWhile) {
            return self.parse_while_stmt(start);
        } else if self.eat(TokenKind::KwFor) {
            return self.parse_for_stmt(start);
        } else if self.is_match_kw() {
            return self.parse_match_stmt(start);
        } else {
            let expr = self.parse_expr()?;
            self.skip_layout();
            if self.eat(TokenKind::Assign) {
                self.skip_layout();
                let value = self.parse_expr()?;
                let target = self.expr_to_pat(expr)?;
                StmtKind::Assign {
                    targets: vec![target],
                    value,
                }
            } else {
                StmtKind::Expr(expr)
            }
        };

        let span = self.span_from(start);
        Some(self.arena.alloc_stmt(kind, span))
    }

    fn parse_if_stmt(&mut self, start: Span) -> Option<StmtId> {
        self.skip_layout();
        let test = self.parse_expr()?;
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' after if condition");
            return None;
        }
        let then_body = self.parse_block()?;
        let mut elifs = Vec::new();
        while self.eat(TokenKind::KwElif) {
            let elif_start = self.current().span;
            self.skip_layout();
            let elif_test = self.parse_expr()?;
            self.skip_layout();
            if !self.eat(TokenKind::Colon) {
                self.error(self.current().span, "expected ':' after elif condition");
                return None;
            }
            let elif_body = self.parse_block()?;
            elifs.push(ElifArm {
                test: elif_test,
                body: elif_body,
                span: self.span_from(elif_start),
            });
        }
        let else_body = if self.eat(TokenKind::KwElse) {
            self.skip_layout();
            if !self.eat(TokenKind::Colon) {
                self.error(self.current().span, "expected ':' after else");
                return None;
            }
            Some(self.parse_block()?)
        } else {
            None
        };
        let span = self.span_from(start);
        Some(self.arena.alloc_stmt(
            StmtKind::If {
                test,
                then_body,
                elifs,
                else_body,
            },
            span,
        ))
    }

    fn parse_while_stmt(&mut self, start: Span) -> Option<StmtId> {
        self.skip_layout();
        let test = self.parse_expr()?;
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' after while condition");
            return None;
        }
        let body = self.parse_block()?;
        let span = self.span_from(start);
        Some(self.arena.alloc_stmt(StmtKind::While { test, body }, span))
    }

    fn parse_for_stmt(&mut self, start: Span) -> Option<StmtId> {
        self.skip_layout();
        let pat = self.parse_pat()?;
        self.skip_layout();
        if !self.eat(TokenKind::KwIn) {
            self.error(self.current().span, "expected 'in' in for loop");
            return None;
        }
        self.skip_layout();
        let iter = self.parse_expr()?;
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' after for header");
            return None;
        }
        let body = self.parse_block()?;
        let span = self.span_from(start);
        Some(self.arena.alloc_stmt(
            StmtKind::For { pat, iter, body },
            span,
        ))
    }

    fn parse_match_stmt(&mut self, start: Span) -> Option<StmtId> {
        self.bump();
        self.skip_layout();
        let scrutinee = self.parse_expr()?;
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' after match scrutinee");
            return None;
        }
        if !self.eat(TokenKind::Newline) {
            self.error(self.current().span, "expected newline after match ':'");
            return None;
        }
        if !self.eat(TokenKind::Indent) {
            self.error(self.current().span, "expected indented match arms");
            return None;
        }
        let mut arms = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
            self.skip_stmt_separators();
            if matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
                break;
            }
            let arm_start = self.current().span;
            let pat = self.parse_pat()?;
            if !self.eat(TokenKind::FatArrow) {
                self.error(self.current().span, "expected '=>' in match arm");
                return None;
            }
            let body = self.parse_block()?;
            arms.push(MatchArm {
                pat,
                guard: None,
                body,
                span: self.span_from(arm_start),
            });
            self.skip_stmt_separators();
        }
        self.eat(TokenKind::Dedent);
        let span = self.span_from(start);
        Some(self.arena.alloc_stmt(
            StmtKind::Match { scrutinee, arms },
            span,
        ))
    }

    pub fn parse_block(&mut self) -> Option<Vec<StmtId>> {
        if !self.eat(TokenKind::Newline) {
            let stmt = self.parse_stmt()?;
            return Some(vec![stmt]);
        }
        if !self.eat(TokenKind::Indent) {
            self.error(self.current().span, "expected indented block");
            return None;
        }
        let mut stmts = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
            self.skip_stmt_separators();
            if matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
                break;
            }
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else if self.handler.has_errors() {
                self.synchronize();
            } else {
                break;
            }
        }
        self.eat(TokenKind::Dedent);
        Some(stmts)
    }

    pub fn parse_pat(&mut self) -> Option<PatId> {
        let start = self.current().span;
        if self.eat(TokenKind::Underscore) {
            let span = self.span_from(start);
            return Some(self.arena.alloc_pat(PatKind::Wild, span));
        }
        if let TokenKind::Ident { name } = self.current_kind() {
            if name == "_" {
                self.bump();
                let span = self.span_from(start);
                return Some(self.arena.alloc_pat(PatKind::Wild, span));
            }
        }
        if matches!(
            self.current_kind(),
            TokenKind::IntLit { .. }
                | TokenKind::FloatLit { .. }
                | TokenKind::StringLit { .. }
                | TokenKind::BoolLit(_)
                | TokenKind::KwTrue
                | TokenKind::KwFalse
                | TokenKind::KwNone
        ) {
            let expr = self.parse_expr()?;
            return self.expr_to_pat(expr);
        }
        if self.eat(TokenKind::LParen) {
            self.skip_layout();
            let mut pats = Vec::new();
            if !self.eat(TokenKind::RParen) {
                pats.push(self.parse_pat()?);
                while self.eat(TokenKind::Comma) {
                    self.skip_layout();
                    if self.eat(TokenKind::RParen) {
                        break;
                    }
                    pats.push(self.parse_pat()?);
                }
                if !self.expect(TokenKind::RParen, "expected ')'") {
                    return None;
                }
            }
            let span = self.span_from(start);
            if pats.len() == 1 {
                return Some(pats[0]);
            }
            return Some(self.arena.alloc_pat(PatKind::Tuple(pats), span));
        }

        let path = self.parse_path()?;
        if self.eat(TokenKind::LParen) {
            let variant = path.segments.last().unwrap().ident.clone();
            let mut subpats = Vec::new();
            if !self.eat(TokenKind::RParen) {
                loop {
                    subpats.push(self.parse_pat()?);
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
            let span = self.span_from(start);
            return Some(self.arena.alloc_pat(
                PatKind::Enum {
                    path,
                    variant,
                    subpats,
                },
                span,
            ));
        }

        let name = path.segments.last()?.ident.clone();
        let span = self.span_from(start);
        Some(self.arena.alloc_pat(
            PatKind::Ident {
                name,
                mutability: PatMutability::Imm,
                subpat: None,
            },
            span,
        ))
    }

    fn expr_to_pat(&mut self, expr: ExprId) -> Option<PatId> {
        let expr_node = self.arena.expr(expr);
        let span = expr_node.span;
        let kind = match &expr_node.kind {
            rpython_ast::ExprKind::Path(path) if path.segments.len() == 1 => PatKind::Ident {
                name: path.segments[0].ident.clone(),
                mutability: PatMutability::Imm,
                subpat: None,
            },
            rpython_ast::ExprKind::Literal(lit) => PatKind::Literal(lit.clone()),
            _ => {
                self.error(span, "invalid assignment target");
                return None;
            }
        };
        Some(self.arena.alloc_pat(kind, span))
    }
}
