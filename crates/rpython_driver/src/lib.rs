mod link;
mod pipeline;
mod session;

pub use session::{CompileOptions, CompilerSession, EmitStage, OptLevel};

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rpython_ast::Arena;
use rpython_codegen_llvm::compile_crate_to_c;
use rpython_errors::{Handler, HumanEmitter};
use rpython_mir::interp::interpret_crate;
use rpython_resolve::resolve_crate;
use rpython_span::SourceMap;
use rpython_syntax::{tokenize, SpannedToken, TokenKind};
use rpython_typeck::typecheck;

use pipeline::{emit_ast, emit_mir, run_pipeline, CompiledUnit};

/// Compile a source file through the full pipeline.
pub fn compile(session: &mut CompilerSession) -> Result<()> {
    match session.options.emit {
        EmitStage::Tokens => {
            let unit = run_pipeline(session)?;
            if let Some(tokens) = unit.tokens {
                print!("{tokens}");
            }
            Ok(())
        }
        EmitStage::Ast => {
            let unit = run_pipeline(session)?;
            println!("{}", emit_ast(&unit));
            Ok(())
        }
        EmitStage::Mir => {
            let unit = run_pipeline(session)?;
            println!("{}", emit_mir(&unit));
            Ok(())
        }
        EmitStage::Hir | EmitStage::Llvm => {
            anyhow::bail!("emit stage {:?} not implemented yet", session.options.emit);
        }
        EmitStage::Executable => {
            let unit = run_pipeline(session)?;
            if session.options.run_interp {
                return Ok(());
            }
            let output = session
                .options
                .output
                .clone()
                .unwrap_or_else(|| default_output_path(&session.input_path));
            let mir = unit.mir.as_ref().context("MIR not produced")?;
            let mut handler = Handler::new();
            let resolution = resolve_crate(&unit.module, &unit.arena, &mut handler)
                .context("name resolution failed")?;
            let typed = typecheck(&resolution, &unit.module, &unit.arena, &mut handler)
                .context("typecheck failed")?;
            let c_out = compile_crate_to_c(mir, &typed, &resolution);
            link::link_executable(&c_out, &output, session.options.opt_level)?;
            Ok(())
        }
    }
}

/// Compile `path` to a native executable at `output`.
pub fn compile_to_executable(
    path: &Path,
    output: &Path,
    options: &CompileOptions,
) -> Result<PathBuf> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut source_map = SourceMap::new();
    let file_id = source_map.load_file(path, contents.clone());
    let mut handler = Handler::new();
    let mut session = CompilerSession::new(
        source_map,
        file_id,
        handler,
        options.clone(),
        path.to_path_buf(),
        contents,
    );

    let unit = run_pipeline(&mut session)?;
    let mir = unit
        .mir
        .as_ref()
        .context("MIR not produced")?;

    let resolution = resolve_crate(&unit.module, &unit.arena, &mut session.handler)
        .context("name resolution failed")?;
    let typed = typecheck(&resolution, &unit.module, &unit.arena, &mut session.handler)
        .context("typecheck failed")?;

    let c_out = compile_crate_to_c(mir, &typed, &resolution);
    link::link_executable(&c_out, output, options.opt_level)?;
    Ok(output.to_path_buf())
}

/// Run MIR interpreter on a source file (no codegen).
pub fn run_interpreted(path: &Path) -> Result<()> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut source_map = SourceMap::new();
    let file_id = source_map.load_file(path, contents.clone());
    let mut session = CompilerSession::new(
        source_map,
        file_id,
        Handler::new(),
        CompileOptions {
            run_interp: true,
            emit: EmitStage::Mir,
            ..Default::default()
        },
        path.to_path_buf(),
        contents,
    );
    run_pipeline(&mut session)?;
    Ok(())
}

pub fn load_and_tokenize(path: &Path) -> Result<(SourceMap, String)> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut source_map = SourceMap::new();
    let file_id = source_map.load_file(path, contents);
    let mut handler = Handler::new();
    let stream = tokenize(&source_map, file_id, &mut handler);
    if handler.has_errors() {
        let mut emitter = HumanEmitter::new();
        handler.report(&source_map, &mut emitter);
        anyhow::bail!("{}", emitter.into_string());
    }
    Ok((source_map, format_token_stream(stream.tokens())))
}

fn default_output_path(input: &Path) -> PathBuf {
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("a");
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    parent.join(stem)
}

fn format_token_stream(tokens: &[SpannedToken]) -> String {
    let mut out = String::new();
    for token in tokens {
        out.push_str(&format_token(token));
        out.push('\n');
    }
    out
}

fn format_token(token: &SpannedToken) -> String {
    match &token.kind {
        TokenKind::Eof => "Eof".into(),
        TokenKind::Newline => "Newline".into(),
        TokenKind::Indent => "Indent".into(),
        TokenKind::Dedent => "Dedent".into(),
        TokenKind::IntLit { value } => format!("IntLit({value:?})"),
        TokenKind::FloatLit { value } => format!("FloatLit({value})"),
        TokenKind::StringLit { value } => format!("StringLit({value:?})"),
        TokenKind::BytesLit { value } => format!("BytesLit({value:?})"),
        TokenKind::BoolLit(b) => format!("BoolLit({b})"),
        TokenKind::Ident { name } => format!("Ident({name})"),
        kind => kind.name().to_string(),
    }
}
