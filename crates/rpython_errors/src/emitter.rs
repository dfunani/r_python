use rpython_span::{LineCol, SourceMap};

use crate::{Diagnostic, Handler};

pub trait Emitter {
    fn emit(&mut self, source_map: &SourceMap, diagnostic: &Diagnostic);
    fn finish(&mut self, source_map: &SourceMap, handler: &Handler);
}

pub struct HumanEmitter {
    output: String,
}

impl HumanEmitter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn into_string(self) -> String {
        self.output
    }
}

impl Default for HumanEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Emitter for HumanEmitter {
    fn emit(&mut self, source_map: &SourceMap, diagnostic: &Diagnostic) {
        let level = match diagnostic.level {
            crate::Level::Error => "error",
            crate::Level::Warning => "warning",
            crate::Level::Note => "note",
            crate::Level::Help => "help",
        };
        if let Some(code) = diagnostic.code {
            self.output.push_str(&format!(
                "{level}[{}]: {}\n",
                code.as_str(),
                diagnostic.message
            ));
        } else {
            self.output
                .push_str(&format!("{level}: {}\n", diagnostic.message));
        }
        for label in &diagnostic.labels {
            let file = source_map.file(label.span.file_id).expect("unknown file");
            let LineCol { line, col } = source_map.line_col(label.span);
            self.output
                .push_str(&format!("  --> {}:{}:{}\n", file.name.display(), line, col));
            self.output.push_str(&format!("   | {}\n", label.message));
        }
    }

    fn finish(&mut self, _source_map: &SourceMap, handler: &Handler) {
        if handler.has_errors() {
            self.output
                .push_str(&format!("\n{} error(s) emitted\n", handler.errors()));
        }
    }
}
