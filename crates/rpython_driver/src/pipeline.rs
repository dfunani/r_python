use rpython_ast::{format_module, Arena, Module};
use rpython_borrowck::borrowck_crate;
use rpython_errors::Handler;
use rpython_hir_build::build_hir;
use rpython_mir::{format_mir_crate, interp::interpret_crate, MirCrate};
use rpython_mir_build::build_mir;
use rpython_parse::parse_module;
use rpython_resolve::resolve_crate;
use rpython_syntax::{tokenize, TokenStream};
use rpython_typeck::typecheck;

use crate::session::CompilerSession;

pub struct CompiledUnit {
    pub module: Module,
    pub arena: Arena,
    pub mir: Option<MirCrate>,
    pub tokens: Option<String>,
}

pub fn run_pipeline(session: &mut CompilerSession) -> anyhow::Result<CompiledUnit> {
    let mut handler = std::mem::take(&mut session.handler);

    let stream = tokenize(&session.source_map, session.file_id, &mut handler);
    if handler.has_errors() {
        return Err(report_errors(session, handler));
    }

    let token_text = format_token_stream(stream.tokens());

    if session.options.emit == crate::EmitStage::Tokens {
        session.handler = handler;
        return Ok(CompiledUnit {
            module: Module {
                items: vec![],
                span: rpython_span::Span::dummy(),
            },
            arena: Arena::new(),
            mir: None,
            tokens: Some(token_text),
        });
    }

    let mut arena = Arena::new();
    let module = parse_module(stream, &arena, &mut handler)
        .ok_or_else(|| anyhow::anyhow!("parse failed"))?;
    if handler.has_errors() {
        return Err(report_errors(session, handler));
    }

    if session.options.emit == crate::EmitStage::Ast {
        session.handler = handler;
        return Ok(CompiledUnit {
            module,
            arena,
            mir: None,
            tokens: None,
        });
    }

    let resolution = resolve_crate(&module, &arena, &mut handler)
        .ok_or_else(|| anyhow::anyhow!("name resolution failed"))?;
    if handler.has_errors() {
        return Err(report_errors(session, handler));
    }

    let typed = match typecheck(&resolution, &module, &arena, &mut handler) {
        Some(t) => t,
        None => return Err(report_errors(session, handler)),
    };
    if handler.has_errors() {
        return Err(report_errors(session, handler));
    }

    let hir = build_hir(&typed, &module, &arena);
    let mir = borrowck_crate(build_mir(&hir));

    if session.options.run_interp {
        interpret_crate(&mir).map_err(|e| anyhow::anyhow!("{e}"))?;
    }

    session.handler = handler;
    Ok(CompiledUnit {
        module,
        arena,
        mir: Some(mir),
        tokens: None,
    })
}

fn format_token_stream(tokens: &[rpython_syntax::SpannedToken]) -> String {
    use rpython_syntax::TokenKind::*;
    let mut out = String::new();
    for token in tokens {
        let line = match &token.kind {
            Eof => "Eof".into(),
            Newline => "Newline".into(),
            Indent => "Indent".into(),
            Dedent => "Dedent".into(),
            IntLit { value } => format!("IntLit({value:?})"),
            FloatLit { value } => format!("FloatLit({value})"),
            StringLit { value } => format!("StringLit({value:?})"),
            BytesLit { value } => format!("BytesLit({value:?})"),
            BoolLit(b) => format!("BoolLit({b})"),
            Ident { name } => format!("Ident({name})"),
            kind => kind.name().to_string(),
        };
        out.push_str(&line);
        out.push('\n');
    }
    out
}

fn report_errors(session: &CompilerSession, handler: Handler) -> anyhow::Error {
    let mut emitter = rpython_errors::HumanEmitter::new();
    handler.report(&session.source_map, &mut emitter);
    anyhow::anyhow!("{}", emitter.into_string())
}

pub fn emit_ast(unit: &CompiledUnit) -> String {
    format_module(&unit.module, &unit.arena)
}

pub fn emit_mir(unit: &CompiledUnit) -> String {
    unit
        .mir
        .as_ref()
        .map(format_mir_crate)
        .unwrap_or_default()
}
