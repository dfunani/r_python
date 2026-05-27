use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use rpython_driver::{CompileOptions, CompilerSession, EmitStage, OptLevel};
use rpython_errors::{ErrorCode, Handler, HumanEmitter};
use rpython_span::SourceMap;
use rpython_syntax::tokenize;

#[derive(Parser, Debug)]
#[command(name = "rpythonc", version, about = "rPython compiler driver")]
struct Args {
    /// Input `.rpy` source file
    input: PathBuf,

    /// Output executable path
    #[arg(short = 'o')]
    output: Option<PathBuf>,

    /// Stop after a compiler stage and print intermediate output
    #[arg(long, value_enum)]
    emit: Option<EmitKind>,

    /// Run via MIR interpreter (no codegen)
    #[arg(long)]
    run: bool,

    /// Optimization level (LLVM/C backend)
    #[arg(long, default_value = "0")]
    opt: u8,

    /// Print documentation for an error code (e.g. E0001)
    #[arg(long)]
    explain: Option<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum EmitKind {
    Tokens,
    Ast,
    Hir,
    Mir,
    Llvm,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    if let Some(code) = args.explain {
        println!("{}", explain_error(&code));
        return Ok(());
    }

    let emit = match args.emit {
        Some(EmitKind::Tokens) => EmitStage::Tokens,
        Some(EmitKind::Ast) => EmitStage::Ast,
        Some(EmitKind::Hir) => EmitStage::Hir,
        Some(EmitKind::Mir) => EmitStage::Mir,
        Some(EmitKind::Llvm) => EmitStage::Llvm,
        None if args.run => EmitStage::Executable,
        None => EmitStage::Executable,
    };

    let opt_level = match args.opt {
        0 => OptLevel::O0,
        1 => OptLevel::O1,
        2 => OptLevel::O2,
        3 => OptLevel::O3,
        n => bail!("unsupported optimization level {n} (use 0-3)"),
    };

    let contents = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;
    let mut source_map = SourceMap::new();
    let file_id = source_map.load_file(&args.input, contents.clone());

    let mut handler = Handler::new();
    if emit == EmitStage::Tokens {
        let stream = tokenize(&source_map, file_id, &mut handler);
        if handler.has_errors() {
            let mut emitter = HumanEmitter::new();
            handler.report(&source_map, &mut emitter);
            bail!("{}", emitter.into_string());
        }
        for token in stream {
            println!("{}", format_token(&token));
        }
        return Ok(());
    }

    let options = CompileOptions {
        emit,
        output: args.output.clone(),
        opt_level,
        run_interp: args.run,
    };

    let mut session = CompilerSession::new(
        source_map,
        file_id,
        handler,
        options,
        args.input.clone(),
        contents,
    );
    rpython_driver::compile(&mut session)?;
    Ok(())
}

fn explain_error(code: &str) -> &'static str {
    let num = code
        .strip_prefix('E')
        .or_else(|| code.strip_prefix('e'))
        .and_then(|s| s.parse::<u16>().ok());
    match num {
        Some(n) => ErrorCode(n).explain(),
        None => "expected error code like E0001",
    }
}

fn format_token(token: &rpython_syntax::SpannedToken) -> String {
    use rpython_syntax::TokenKind::*;
    match &token.kind {
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
    }
}
