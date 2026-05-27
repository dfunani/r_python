use std::path::PathBuf;

use rpython_errors::Handler;
use rpython_span::{FileId, SourceMap};

/// Options for a single compile or run invocation.
#[derive(Clone, Debug)]
pub struct CompileOptions {
    pub emit: CompilationStage,
    pub output: Option<PathBuf>,
    pub opt_level: OptLevel,
    pub run_interpreter: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            emit: CompilationStage::default(),
            output: None,
            opt_level: OptLevel::O0,
            run_interpreter: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OptLevel {
    #[default]
    O0,
    O1,
    O2,
    O3,
}

/// Compiler stage at which to stop and print output (if any).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CompilationStage {
    LexerTokens,
    AbstractSyntaxTree,
    HighLevelIntermediateRepresentation,
    MidLevelIntermediateRepresentation,
    LlvmIntermediateRepresentation,
    #[default]
    NativeExecutable,
}

/// Deprecated alias — prefer [`CompilationStage`].
pub type EmitStage = CompilationStage;

#[derive(Debug)]
pub struct CompilerSession {
    pub source_map: SourceMap,
    pub file_id: FileId,
    pub handler: Handler,
    pub options: CompileOptions,
    pub input_path: PathBuf,
    pub source: String,
}

impl CompilerSession {
    pub fn new(
        source_map: SourceMap,
        file_id: FileId,
        handler: Handler,
        options: CompileOptions,
        input_path: PathBuf,
        source: String,
    ) -> Self {
        Self {
            source_map,
            file_id,
            handler,
            options,
            input_path,
            source,
        }
    }
}
