use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use rpython_codegen_llvm::COutput;
use rpython_runtime::runtime_c_src;

use crate::session::OptLevel;

fn opt_flag(opt: OptLevel) -> &'static str {
    match opt {
        OptLevel::O0 => "-O0",
        OptLevel::O1 => "-O1",
        OptLevel::O2 => "-O2",
        OptLevel::O3 => "-O3",
    }
}

/// Compile generated C plus runtime and link an executable.
pub fn link_executable(
    c_output: &COutput,
    output: &Path,
    opt: OptLevel,
) -> anyhow::Result<PathBuf> {
    let dir = tempfile::tempdir().context("create temp dir")?;
    let c_path = dir.path().join("out.c");
    std::fs::write(&c_path, &c_output.source).context("write generated C")?;

    let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let status = Command::new(&compiler)
        .arg("-std=c11")
        .arg(opt_flag(opt))
        .arg(&c_path)
        .arg(runtime_c_src())
        .arg("-o")
        .arg(output)
        .status()
        .with_context(|| format!("invoke C compiler `{compiler}`"))?;

    if !status.success() {
        anyhow::bail!("C compiler failed with status {status}");
    }
    Ok(output.to_path_buf())
}
