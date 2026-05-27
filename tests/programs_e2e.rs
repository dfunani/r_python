//! End-to-end: compile and run examples/hello.rpy as a native binary.

use std::process::Command;

use rpython_driver::{compile_to_executable, CompileOptions, EmitStage, OptLevel};

#[test]
fn hello_interpret_and_native() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let hello = manifest.join("examples/hello.rpy");

    let out_dir = std::env::temp_dir().join(format!("rpython_e2e_{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).unwrap();
    let bin = out_dir.join("hello_e2e");

    compile_to_executable(
        &hello,
        &bin,
        &CompileOptions {
            emit: EmitStage::Executable,
            output: Some(bin.clone()),
            opt_level: OptLevel::O0,
            run_interp: false,
        },
    )
    .expect("compile hello.rpy");

    let output = Command::new(&bin)
        .output()
        .expect("run hello binary");
    assert!(output.status.success(), "hello binary failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("hello, rPython"),
        "expected greeting, got: {stdout}"
    );
}
