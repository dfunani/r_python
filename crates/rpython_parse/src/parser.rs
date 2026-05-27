use rpython_ast::Arena;
use rpython_errors::{Diagnostic, Handler};
use rpython_span::Span;
use rpython_syntax::{SpannedToken, TokenKind};

pub struct Parser<'a> {
    pub(crate) tokens: &'a [SpannedToken],
    pub(crate) pos: usize,
    pub(crate) arena: &'a mut Arena,
    pub(crate) handler: &'a mut Handler,
}

impl<'a> Parser<'a> {
    pub fn new(
        tokens: &'a [SpannedToken],
        arena: &'a mut Arena,
        handler: &'a mut Handler,
    ) -> Self {
        Self {
            tokens,
            pos: 0,
            arena,
            handler,
        }
    }

    pub fn is_at_end(&self) -> bool {
        matches!(self.current_kind(), TokenKind::Eof)
    }

    pub fn current(&self) -> &SpannedToken {
        &self.tokens[self.pos]
    }

    pub fn current_kind(&self) -> &TokenKind {
        &self.current().kind
    }

    pub fn peek(&self, offset: usize) -> Option<&SpannedToken> {
        self.tokens.get(self.pos + offset)
    }

    pub fn peek_kind(&self, offset: usize) -> Option<&TokenKind> {
        self.peek(offset).map(|t| &t.kind)
    }

    pub fn bump(&mut self) -> &SpannedToken {
        let token = &self.tokens[self.pos];
        if !matches!(token.kind, TokenKind::Eof) {
            self.pos += 1;
        }
        token
    }

    pub fn span_from(&self, start: Span) -> Span {
        let end = self.tokens[self.pos.saturating_sub(1)].span;
        Span::merge(start, end)
    }

    pub fn error(&mut self, span: Span, message: impl Into<String>) {
        self.handler.emit(
            Diagnostic::error(message)
                .with_label(span, "here", true),
        );
    }

    pub fn expect(&mut self, kind: TokenKind, message: &str) -> bool {
        if std::mem::discriminant(self.current_kind()) == std::mem::discriminant(&kind) {
            self.bump();
            true
        } else {
            self.error(self.current().span, message);
            false
        }
    }

    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if std::mem::discriminant(self.current_kind()) == std::mem::discriminant(&kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn eat_ident(&mut self, name: &str) -> bool {
        if let TokenKind::Ident { name: ident } = self.current_kind() {
            if ident.as_str() == name {
                self.bump();
                return true;
            }
        }
        false
    }

    pub fn skip_layout(&mut self) {
        while matches!(self.current_kind(), TokenKind::Newline) {
            self.bump();
        }
    }

    pub fn skip_stmt_separators(&mut self) {
        while matches!(self.current_kind(), TokenKind::Newline) {
            self.bump();
        }
    }

    pub fn at_stmt_start(&self) -> bool {
        matches!(
            self.current_kind(),
            TokenKind::KwDef
                | TokenKind::KwClass
                | TokenKind::KwStruct
                | TokenKind::KwEnum
                | TokenKind::KwImpl
                | TokenKind::KwInterface
                | TokenKind::KwTrait
                | TokenKind::KwImport
                | TokenKind::KwFrom
                | TokenKind::KwIf
                | TokenKind::KwWhile
                | TokenKind::KwFor
                | TokenKind::KwReturn
                | TokenKind::KwPass
                | TokenKind::KwBreak
                | TokenKind::KwContinue
                | TokenKind::KwPub
                | TokenKind::At
                | TokenKind::Dedent
                | TokenKind::Eof
        ) || self.is_match_kw()
            || matches!(
                self.current_kind(),
                TokenKind::Ident { .. }
                    | TokenKind::IntLit { .. }
                    | TokenKind::FloatLit { .. }
                    | TokenKind::StringLit { .. }
                    | TokenKind::BytesLit { .. }
                    | TokenKind::BoolLit(_)
                    | TokenKind::KwTrue
                    | TokenKind::KwFalse
                    | TokenKind::KwNone
                    | TokenKind::KwSelf
                    | TokenKind::LParen
                    | TokenKind::LBracket
                    | TokenKind::LBrace
                    | TokenKind::Minus
                    | TokenKind::Bang
            )
    }

    pub fn is_match_kw(&self) -> bool {
        matches!(self.current_kind(), TokenKind::Ident { name } if name == "match")
    }

    pub fn synchronize(&mut self) {
        while !self.is_at_end() {
            if self.at_stmt_start() {
                return;
            }
            self.bump();
        }
    }
}
