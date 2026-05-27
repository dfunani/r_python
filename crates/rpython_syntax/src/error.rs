use rpython_span::Span;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum LexError {
    #[error("invalid character `{ch}`")]
    InvalidChar { ch: char, span: Span },
    #[error("unterminated string literal")]
    UnterminatedString { span: Span },
    #[error("invalid number literal")]
    InvalidNumber { span: Span },
    #[error("inconsistent indentation")]
    InconsistentIndent { span: Span },
}
