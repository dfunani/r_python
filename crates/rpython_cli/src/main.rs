use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use rpython_driver::{CompilationStage, CompileOptions, CompilerSession, OptLevel};
use rpython_errors::{ErrorCode, Handler, HumanEmitter};
use rpython_span::SourceMap;
use rpython_syntax::tokenize;

#[derive(Parser, Debug)]
#[command(
    name = "rpythonc",
    version,
    about = "rPython compiler — a compiled language with Python-shaped syntax",
    long_about = "rPython is a statically typed, compiled language (not interpreted like CPython).\n\
                  \n\
                  Typical workflow:\n\
                    rpythonc run hello.rpy          # fast interpreter (development)\n\
                    rpythonc build -o hello hello.rpy   # native executable (production)\n\
                  \n\
                  Install binaries: https://github.com/dfunani/r_python/releases",
    after_help = "Unlike Python, rPython requires compilation for native speed.\n\
                  Use `run` while developing; use `build` to ship a binary."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Legacy: source file when no subcommand is used (same as `build` or `run` with flags)
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    #[arg(short = 'r', long, help = "Run via MIR interpreter (legacy; prefer: rpythonc run FILE)")]
    run: bool,

    #[arg(short = 'o', long, help = "Output executable (legacy; prefer: rpythonc build -o OUT FILE)")]
    output: Option<PathBuf>,

    #[arg(long, value_enum, help = "Stop after a compiler stage")]
    emit: Option<EmitStageArg>,

    #[arg(long, default_value = "0")]
    opt: u8,

    #[arg(long)]
    test: bool,

    #[arg(long)]
    explain: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run a program in the mid-level IR interpreter (no native codegen)
    Run {
        file: PathBuf,
    },
    /// Compile to a native executable (requires a system C compiler)
    Build {
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long, default_value = "0")]
        opt: u8,
        #[arg(long, value_enum)]
        emit: Option<EmitStageArg>,
    },
    /// Run `#[test]` functions in a source file
    Test {
        file: PathBuf,
    },
    /// Print documentation for a diagnostic error code
    Explain {
        code: String,
    },
    /// Emit lexer tokens only
    Tokens {
        file: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum EmitStageArg {
    Tokens,
    #[value(name = "ast", alias = "abstract-syntax-tree")]
    AbstractSyntaxTree,
    #[value(name = "high-level-ir", alias = "hir")]
    HighLevelIntermediateRepresentation,
    #[value(name = "mid-level-ir", alias = "mir")]
    MidLevelIntermediateRepresentation,
    #[value(name = "llvm", alias = "llvm-ir")]
    LlvmIntermediateRepresentation,
}

impl From<EmitStageArg> for CompilationStage {
    fn from(value: EmitStageArg) -> Self {
        match value {
            EmitStageArg::Tokens => CompilationStage::LexerTokens,
            EmitStageArg::AbstractSyntaxTree => CompilationStage::AbstractSyntaxTree,
            EmitStageArg::HighLevelIntermediateRepresentation => {
                CompilationStage::HighLevelIntermediateRepresentation
            }
            EmitStageArg::MidLevelIntermediateRepresentation => {
                CompilationStage::MidLevelIntermediateRepresentation
            }
            EmitStageArg::LlvmIntermediateRepresentation => {
                CompilationStage::LlvmIntermediateRepresentation
            }
        }
    }
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
    let cli = Cli::parse();

    if let Some(code) = cli.explain {
        println!("{}", explain_error(&code));
        return Ok(());
    }

    match cli.command {
        Some(Command::Explain { code }) => {
            println!("{}", explain_error(&code));
            Ok(())
        }
        Some(Command::Test { file }) => run_tests(&file),
        Some(Command::Tokens { file }) => emit_tokens(&file),
        Some(Command::Run { file }) => run_interpreted(&file),
        Some(Command::Build {
            file,
            output,
            opt,
            emit,
        }) => compile_file(
            &file,
            output,
            opt,
            emit,
            false,
        ),
        None => {
            let input = cli
                .input
                .context("missing input file — try `rpythonc --help`")?;
            if cli.test {
                return run_tests(&input);
            }
            if let Some(emit) = cli.emit {
                return compile_file(&input, cli.output, cli.opt, Some(emit), cli.run);
            }
            if cli.run {
                return run_interpreted(&input);
            }
            compile_file(&input, cli.output, cli.opt, None, false)
        }
    }
}

fn run_tests(path: &PathBuf) -> Result<()> {
    let report = rpython_test_runner::run_tests(path)?;
    println!(
        "test result: {} passed; {} failed",
        report.passed, report.failed
    );
    for failure in &report.failures {
        eprintln!("FAILED {} — {}", failure.name, failure.message);
    }
    if report.failed > 0 {
        bail!("{} test(s) failed", report.failed);
    }
    Ok(())
}

fn emit_tokens(path: &PathBuf) -> Result<()> {
    let contents = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut source_map = SourceMap::new();
    let file_id = source_map.load_file(path, contents);
    let mut handler = Handler::new();
    let stream = tokenize(&source_map, file_id, &mut handler);
    if handler.has_errors() {
        let mut emitter = HumanEmitter::new();
        handler.report(&source_map, &mut emitter);
        bail!("{}", emitter.into_string());
    }
    for token in stream {
        println!("{}", format_token(&token));
    }
    Ok(())
}

fn run_interpreted(path: &PathBuf) -> Result<()> {
    rpython_driver::run_interpreted(path)
}

fn compile_file(
    path: &PathBuf,
    output: Option<PathBuf>,
    opt: u8,
    emit: Option<EmitStageArg>,
    run_interpreter: bool,
) -> Result<()> {
    let emit_stage = match emit {
        Some(e) => e.into(),
        None if run_interpreter => CompilationStage::NativeExecutable,
        None => CompilationStage::NativeExecutable,
    };

    let opt_level = match opt {
        0 => OptLevel::O0,
        1 => OptLevel::O1,
        2 => OptLevel::O2,
        3 => OptLevel::O3,
        n => bail!("unsupported optimization level {n} (use 0-3)"),
    };

    if emit_stage == CompilationStage::LexerTokens {
        return emit_tokens(path);
    }

    let contents = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut source_map = SourceMap::new();
    let file_id = source_map.load_file(path, contents.clone());

    let options = CompileOptions {
        emit: emit_stage,
        output: output.clone(),
        opt_level,
        run_interpreter,
    };

    let mut session = CompilerSession::new(
        source_map,
        file_id,
        Handler::new(),
        options,
        path.clone(),
        contents,
    );
    rpython_driver::compile(&mut session)
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
