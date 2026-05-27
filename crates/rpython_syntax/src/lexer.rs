use rpython_errors::{Diagnostic, ErrorCode, Handler};
use rpython_span::{BytePos, FileId, Span};
use smol_str::SmolStr;

use crate::error::LexError;
use crate::indent::{column_width, IndentError, IndentStack};
use crate::token::{IntLiteral, SpannedToken, TokenKind};

pub struct TokenStream {
    tokens: Vec<SpannedToken>,
}

impl TokenStream {
    pub fn tokens(&self) -> &[SpannedToken] {
        &self.tokens
    }
}

impl IntoIterator for TokenStream {
    type Item = SpannedToken;
    type IntoIter = std::vec::IntoIter<SpannedToken>;

    fn into_iter(self) -> Self::IntoIter {
        self.tokens.into_iter()
    }
}

pub struct Lexer<'a> {
    file_id: FileId,
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    handler: &'a mut Handler,
    indent_stack: IndentStack,
    at_line_start: bool,
    pending: Vec<SpannedToken>,
    finished: bool,
}

impl<'a> Lexer<'a> {
    pub fn from_source(file_id: FileId, source: &'a str, handler: &'a mut Handler) -> TokenStream {
        let mut lexer = Self {
            file_id,
            source,
            bytes: source.as_bytes(),
            pos: 0,
            handler,
            indent_stack: IndentStack::new(),
            at_line_start: true,
            pending: Vec::new(),
            finished: false,
        };
        let mut tokens = Vec::new();
        while let Some(tok) = lexer.next_token() {
            tokens.push(tok);
        }
        TokenStream { tokens }
    }

    fn next_token(&mut self) -> Option<SpannedToken> {
        if let Some(tok) = self.pending.pop() {
            return Some(tok);
        }
        if self.finished {
            return None;
        }

        self.skip_blank_lines_at_line_start();

        if self.at_line_start {
            if let Err(e) = self.emit_indent_tokens() {
                self.report_lex_error(e);
                if self.handler.should_abort() {
                    self.finished = true;
                    return None;
                }
            }
            self.at_line_start = false;
            if let Some(tok) = self.pending.pop() {
                return Some(tok);
            }
        }

        self.skip_spaces();
        if self.pos >= self.bytes.len() {
            return self.finish_file();
        }

        let start = self.pos;
        let ch = self.peek_char().unwrap();

        if ch == '\n' {
            self.bump();
            self.at_line_start = true;
            return Some(self.make_token(TokenKind::Newline, start, self.pos));
        }

        if ch == '#' {
            self.skip_line_comment();
            return self.next_token();
        }

        let kind = match self.scan_token() {
            Ok(kind) => kind,
            Err(e) => {
                self.report_lex_error(e);
                if self.handler.should_abort() {
                    self.finished = true;
                    return None;
                }
                return self.next_token();
            }
        };
        Some(self.make_token(kind, start, self.pos))
    }

    fn finish_file(&mut self) -> Option<SpannedToken> {
        let dedents = self.indent_stack.dedent_to_zero();
        let span = self.span_at(self.pos, self.pos);
        for _ in 0..dedents {
            self.pending.push(SpannedToken {
                kind: TokenKind::Dedent,
                span,
            });
        }
        self.finished = true;
        if let Some(tok) = self.pending.pop() {
            self.pending.push(SpannedToken {
                kind: TokenKind::Eof,
                span,
            });
            return Some(tok);
        }
        Some(SpannedToken {
            kind: TokenKind::Eof,
            span,
        })
    }

    fn emit_indent_tokens(&mut self) -> Result<(), LexError> {
        let line_start = self.pos;
        let indent_end = self.measure_indent_end();
        let line_prefix = &self.source[line_start..indent_end];
        let level = column_width(line_prefix);
        let span = self.span_at(line_start, indent_end);
        self.pos = indent_end;

        match self.indent_stack.transition(level) {
            Ok((indents, dedents)) => {
                for _ in 0..dedents {
                    self.pending.push(SpannedToken {
                        kind: TokenKind::Dedent,
                        span,
                    });
                }
                for _ in 0..indents {
                    self.pending.push(SpannedToken {
                        kind: TokenKind::Indent,
                        span,
                    });
                }
                Ok(())
            }
            Err(IndentError::Inconsistent) => Err(LexError::InconsistentIndent { span }),
        }
    }

    fn measure_indent_end(&self) -> usize {
        let mut i = self.pos;
        while i < self.bytes.len() {
            match self.bytes[i] as char {
                ' ' | '\t' => i += 1,
                _ => break,
            }
        }
        i
    }

    fn skip_blank_lines_at_line_start(&mut self) {
        while self.at_line_start && self.pos < self.bytes.len() {
            let line_start = self.pos;
            let indent_end = self.measure_indent_end();
            let mut i = indent_end;
            while i < self.bytes.len() && self.bytes[i] != b'\n' {
                if !self.bytes[i].is_ascii_whitespace() {
                    return;
                }
                i += 1;
            }
            let blank = i == indent_end
                || (i > indent_end
                    && self.bytes[indent_end..i]
                        .iter()
                        .all(|b| *b == b' ' || *b == b'\t'));
            if !blank && i > indent_end {
                return;
            }
            if i < self.bytes.len() && self.bytes[i] == b'\n' {
                self.pos = i + 1;
                continue;
            }
            if blank && i >= self.bytes.len() {
                self.pos = line_start;
                return;
            }
            return;
        }
    }

    fn scan_token(&mut self) -> Result<TokenKind, LexError> {
        let ch = self.peek_char().unwrap();
        match ch {
            '(' => {
                self.bump();
                Ok(TokenKind::LParen)
            }
            ')' => {
                self.bump();
                Ok(TokenKind::RParen)
            }
            '[' => {
                self.bump();
                Ok(TokenKind::LBracket)
            }
            ']' => {
                self.bump();
                Ok(TokenKind::RBracket)
            }
            '{' => {
                self.bump();
                Ok(TokenKind::LBrace)
            }
            '}' => {
                self.bump();
                Ok(TokenKind::RBrace)
            }
            ':' => {
                self.bump();
                if self.peek_char() == Some('=') {
                    self.bump();
                    Ok(TokenKind::Assign)
                } else {
                    Ok(TokenKind::Colon)
                }
            }
            ';' => {
                self.bump();
                Ok(TokenKind::Semi)
            }
            ',' => {
                self.bump();
                Ok(TokenKind::Comma)
            }
            '.' => {
                self.bump();
                Ok(TokenKind::Dot)
            }
            '@' => {
                self.bump();
                Ok(TokenKind::At)
            }
            '|' => {
                self.bump();
                Ok(TokenKind::Pipe)
            }
            '?' => {
                self.bump();
                Ok(TokenKind::Question)
            }
            '+' => {
                self.bump();
                Ok(TokenKind::Plus)
            }
            '-' => {
                self.bump();
                if self.peek_char() == Some('>') {
                    self.bump();
                    Ok(TokenKind::Arrow)
                } else {
                    Ok(TokenKind::Minus)
                }
            }
            '*' => {
                self.bump();
                Ok(TokenKind::Star)
            }
            '/' => {
                self.bump();
                if self.peek_char() == Some('/') {
                    self.bump();
                    Ok(TokenKind::FloorDiv)
                } else {
                    Ok(TokenKind::Slash)
                }
            }
            '%' => {
                self.bump();
                Ok(TokenKind::Percent)
            }
            '=' => {
                self.bump();
                if self.peek_char() == Some('=') {
                    self.bump();
                    Ok(TokenKind::EqEq)
                } else if self.peek_char() == Some('>') {
                    self.bump();
                    Ok(TokenKind::FatArrow)
                } else {
                    Ok(TokenKind::Assign)
                }
            }
            '!' => {
                self.bump();
                if self.peek_char() == Some('=') {
                    self.bump();
                    Ok(TokenKind::NotEq)
                } else {
                    Ok(TokenKind::Bang)
                }
            }
            '<' => {
                self.bump();
                if self.peek_char() == Some('=') {
                    self.bump();
                    Ok(TokenKind::LtEq)
                } else {
                    Ok(TokenKind::Lt)
                }
            }
            '>' => {
                self.bump();
                if self.peek_char() == Some('=') {
                    self.bump();
                    Ok(TokenKind::GtEq)
                } else {
                    Ok(TokenKind::Gt)
                }
            }
            '&' => {
                self.bump();
                if self.source[self.pos..].starts_with("mut")
                    && !self
                        .source
                        .get(self.pos + 3..)
                        .and_then(|s| s.chars().next())
                        .is_some_and(is_ident_continue)
                {
                    self.bump_n(3);
                    Ok(TokenKind::AmpMut)
                } else {
                    Ok(TokenKind::Amp)
                }
            }
            '"' => self.scan_string(),
            '\'' => self.scan_string(),
            'b' if self.source[self.pos..].starts_with("b\"")
                || self.source[self.pos..].starts_with("b'") =>
            {
                self.scan_bytes()
            }
            '_' if self.peek_ident_start() => self.scan_ident_or_keyword(),
            '0'..='9' => self.scan_number(),
            ch if is_ident_start(ch) => self.scan_ident_or_keyword(),
            ch => Err(LexError::InvalidChar {
                ch,
                span: self.span_at(self.pos, self.pos + ch.len_utf8()),
            }),
        }
    }

    fn scan_ident_or_keyword(&mut self) -> Result<TokenKind, LexError> {
        let start = self.pos;
        self.bump();
        while self.peek_char().is_some_and(is_ident_continue) {
            self.bump();
        }
        let text = &self.source[start..self.pos];
        if text == "_" {
            return Ok(TokenKind::Underscore);
        }
        Ok(keyword_or_ident(text))
    }

    fn scan_number(&mut self) -> Result<TokenKind, LexError> {
        let start = self.pos;

        if self.peek_char() == Some('0') {
            self.bump();
            if self.peek_char() == Some('x') || self.peek_char() == Some('X') {
                self.bump();
                let digits_start = self.pos;
                while self.peek_char().is_some_and(|c| c.is_ascii_hexdigit()) {
                    self.bump();
                }
                let digits = &self.source[digits_start..self.pos];
                if digits.is_empty() {
                    return Err(LexError::InvalidNumber {
                        span: self.span_at(start, self.pos),
                    });
                }
                let value =
                    i64::from_str_radix(digits, 16).map_err(|_| LexError::InvalidNumber {
                        span: self.span_at(start, self.pos),
                    })?;
                return Ok(TokenKind::IntLit {
                    value: IntLiteral::Hex(value),
                });
            }
            if self.peek_char() == Some('b') || self.peek_char() == Some('B') {
                self.bump();
                let digits_start = self.pos;
                while self.peek_char().is_some_and(|c| c == '0' || c == '1') {
                    self.bump();
                }
                let digits = &self.source[digits_start..self.pos];
                if digits.is_empty() {
                    return Err(LexError::InvalidNumber {
                        span: self.span_at(start, self.pos),
                    });
                }
                let value =
                    i64::from_str_radix(digits, 2).map_err(|_| LexError::InvalidNumber {
                        span: self.span_at(start, self.pos),
                    })?;
                return Ok(TokenKind::IntLit {
                    value: IntLiteral::Bin(value),
                });
            }
        }

        while self.peek_char().is_some_and(|c| c.is_ascii_digit()) {
            self.bump();
        }

        if self.peek_char() == Some('.')
            && self
                .bytes
                .get(self.pos + 1)
                .is_some_and(|b| b.is_ascii_digit())
        {
            self.bump();
            while self.peek_char().is_some_and(|c| c.is_ascii_digit()) {
                self.bump();
            }
            let text = &self.source[start..self.pos];
            let value: f64 = text.parse().map_err(|_| LexError::InvalidNumber {
                span: self.span_at(start, self.pos),
            })?;
            return Ok(TokenKind::FloatLit { value });
        }

        let text = &self.source[start..self.pos];
        let value: i64 = text.parse().map_err(|_| LexError::InvalidNumber {
            span: self.span_at(start, self.pos),
        })?;
        Ok(TokenKind::IntLit {
            value: IntLiteral::Decimal(value),
        })
    }

    fn scan_string(&mut self) -> Result<TokenKind, LexError> {
        let start = self.pos;
        let quote = self.peek_char().unwrap();
        self.bump();
        let mut value = String::new();
        while self.pos < self.bytes.len() {
            let ch = self.peek_char().unwrap();
            if ch == quote {
                self.bump();
                return Ok(TokenKind::StringLit { value });
            }
            if ch == '\\' {
                self.bump();
                let esc = self.peek_char().ok_or(LexError::UnterminatedString {
                    span: self.span_at(start, self.pos),
                })?;
                self.bump();
                let decoded = match esc {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    other => other,
                };
                value.push(decoded);
                continue;
            }
            if ch == '\n' {
                return Err(LexError::UnterminatedString {
                    span: self.span_at(start, self.pos),
                });
            }
            value.push(ch);
            self.bump();
        }
        Err(LexError::UnterminatedString {
            span: self.span_at(start, self.pos),
        })
    }

    fn scan_bytes(&mut self) -> Result<TokenKind, LexError> {
        let start = self.pos;
        self.bump(); // b
        let quote = self.peek_char().ok_or(LexError::UnterminatedString {
            span: self.span_at(start, self.pos),
        })?;
        self.bump();
        let mut value = Vec::new();
        while self.pos < self.bytes.len() {
            let ch = self.peek_char().unwrap();
            if ch == quote {
                self.bump();
                return Ok(TokenKind::BytesLit { value });
            }
            if ch == '\\' {
                self.bump();
                let esc = self.peek_char().ok_or(LexError::UnterminatedString {
                    span: self.span_at(start, self.pos),
                })?;
                self.bump();
                value.push(match esc {
                    'n' => b'\n',
                    't' => b'\t',
                    'r' => b'\r',
                    '\\' => b'\\',
                    other => other as u8,
                });
                continue;
            }
            if ch == '\n' {
                return Err(LexError::UnterminatedString {
                    span: self.span_at(start, self.pos),
                });
            }
            value.push(ch as u8);
            self.bump();
        }
        Err(LexError::UnterminatedString {
            span: self.span_at(start, self.pos),
        })
    }

    fn skip_spaces(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] as char {
                ' ' | '\t' | '\r' => self.bump(),
                _ => break,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
            self.bump();
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.pos >= self.bytes.len() {
            None
        } else {
            Some(self.source[self.pos..].chars().next().unwrap())
        }
    }

    fn peek_ident_start(&self) -> bool {
        self.pos + 1 < self.bytes.len() && is_ident_start(self.bytes[self.pos + 1] as char)
    }

    fn bump(&mut self) {
        if self.pos < self.bytes.len() {
            let ch = self.source[self.pos..].chars().next().unwrap();
            self.pos += ch.len_utf8();
        }
    }

    fn bump_n(&mut self, n: usize) {
        for _ in 0..n {
            self.bump();
        }
    }

    fn make_token(&self, kind: TokenKind, start: usize, end: usize) -> SpannedToken {
        SpannedToken {
            kind,
            span: self.span_at(start, end),
        }
    }

    fn span_at(&self, start: usize, end: usize) -> Span {
        Span::new(self.file_id, BytePos(start as u32), BytePos(end as u32))
    }

    fn report_lex_error(&mut self, error: LexError) {
        let (message, code, span) = match &error {
            LexError::InvalidChar { ch, span } => {
                (format!("invalid character `{ch}`"), ErrorCode::E0001, *span)
            }
            LexError::UnterminatedString { span } => (
                "unterminated string literal".into(),
                ErrorCode::E0002,
                *span,
            ),
            LexError::InvalidNumber { span } => {
                ("invalid number literal".into(), ErrorCode::E0001, *span)
            }
            LexError::InconsistentIndent { span } => {
                ("inconsistent indentation".into(), ErrorCode::E0001, *span)
            }
        };
        self.handler.emit(
            Diagnostic::error(message)
                .with_code(code)
                .with_label(span, "here", true),
        );
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn keyword_or_ident(text: &str) -> TokenKind {
    use TokenKind::*;
    match text {
        "def" => KwDef,
        "class" => KwClass,
        "enum" => KwEnum,
        "struct" => KwStruct,
        "impl" => KwImpl,
        "interface" => KwInterface,
        "trait" => KwTrait, // deprecated: use `interface`
        "if" => KwIf,
        "elif" => KwElif,
        "else" => KwElse,
        "while" => KwWhile,
        "for" => KwFor,
        "loop" => KwLoop,
        "break" => KwBreak,
        "continue" => KwContinue,
        "return" => KwReturn,
        "pass" => KwPass,
        "import" => KwImport,
        "from" => KwFrom,
        "as" => KwAs,
        "pub" => KwPub,
        "mut" => KwMut,
        "True" => KwTrue,
        "False" => KwFalse,
        "None" => KwNone,
        "self" => KwSelf,
        "in" => KwIn,
        "not" => KwNot,
        "and" => KwAnd,
        "or" => KwOr,
        "extern" => KwExtern,
        "unsafe" => KwUnsafe,
        "true" => BoolLit(true),
        "false" => BoolLit(false),
        _ => TokenKind::Ident {
            name: SmolStr::new(text),
        },
    }
}
