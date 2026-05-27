use crate::{Diagnostic, Emitter, Level};

#[derive(Debug, Default)]
pub struct Handler {
    diagnostics: Vec<Diagnostic>,
    errors: usize,
    warnings: usize,
    max_errors: usize,
}

impl Handler {
    pub fn new() -> Self {
        Self {
            max_errors: 50,
            ..Default::default()
        }
    }

    pub fn emit(&mut self, diagnostic: Diagnostic) {
        match diagnostic.level {
            Level::Error => self.errors += 1,
            Level::Warning => self.warnings += 1,
            Level::Note | Level::Help => {}
        }
        self.diagnostics.push(diagnostic);
    }

    pub fn error(&mut self, span: rpython_span::Span, message: impl Into<String>) {
        self.emit(
            Diagnostic::error(message)
                .with_label(span, "", true),
        );
    }

    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }

    pub fn errors(&self) -> usize {
        self.errors
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn should_abort(&self) -> bool {
        self.errors >= self.max_errors
    }

    pub fn report<E: Emitter>(&self, source_map: &rpython_span::SourceMap, emitter: &mut E) {
        for diagnostic in &self.diagnostics {
            emitter.emit(source_map, diagnostic);
        }
        emitter.finish(source_map, self);
    }
}
