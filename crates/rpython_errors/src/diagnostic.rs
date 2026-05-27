use rpython_span::Span;

use crate::ErrorCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Clone, Debug)]
pub struct Label {
    pub span: Span,
    pub message: String,
    pub primary: bool,
}

#[derive(Clone, Debug)]
pub struct Suggestion {
    pub message: String,
    pub replacement: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub level: Level,
    pub code: Option<ErrorCode>,
    pub message: String,
    pub labels: Vec<Label>,
    pub suggestions: Vec<Suggestion>,
    pub children: Vec<Diagnostic>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: Level::Error,
            code: None,
            message: message.into(),
            labels: Vec::new(),
            suggestions: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn with_code(mut self, code: ErrorCode) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_label(mut self, span: Span, message: impl Into<String>, primary: bool) -> Self {
        self.labels.push(Label {
            span,
            message: message.into(),
            primary,
        });
        self
    }
}
