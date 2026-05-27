//! End-to-end: run v2 example programs via `rpythonc run`.

use std::path::{Path, PathBuf};
use std::process::Command;

use rpython_driver::{compile_to_executable, CompilationStage, CompileOptions, OptLevel};

fn rpythonc_bin() -> PathBuf {
    let bin = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/rpythonc");
    if !bin.exists() {
        let status = Command::new("cargo")
            .args(["build", "-p", "rpython_cli", "-q"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .expect("cargo build rpython_cli");
        assert!(status.success(), "failed to build rpythonc");
    }
    bin
}

fn run_interpret(path: &Path) -> String {
    let output = Command::new(rpythonc_bin())
        .arg("run")
        .arg(path)
        .output()
        .expect("spawn rpythonc");
    assert!(
        output.status.success(),
        "rpythonc run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn hello_interpret_and_native() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let hello = manifest.join("examples/hello.rpy");

    let stdout = run_interpret(&hello);
    assert!(stdout.contains("hello, rPython"), "got: {stdout}");

    let out_dir = std::env::temp_dir().join(format!("rpython_e2e_{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).unwrap();
    let bin = out_dir.join("hello_e2e");

    compile_to_executable(
        &hello,
        &bin,
        &CompileOptions {
            emit: CompilationStage::NativeExecutable,
            output: Some(bin.clone()),
            opt_level: OptLevel::O0,
            run_interpreter: false,
        },
    )
    .expect("compile hello.rpy");

    let output = Command::new(&bin).output().expect("run hello binary");
    assert!(output.status.success(), "hello binary failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("hello, rPython"),
        "expected greeting, got: {stdout}"
    );
}

#[test]
fn static_typing_interpret() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/static_typing.rpy");
    let stdout = run_interpret(&path);
    assert!(
        stdout.contains("hello"),
        "expected str print, got: {stdout}"
    );
    assert!(stdout.contains("42"), "expected int print, got: {stdout}");
}

#[test]
fn gcd_interpret() {
    let gcd = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/gcd.rpy");
    let stdout = run_interpret(&gcd);
    assert!(stdout.contains('6'), "expected gcd(48,18)=6, got: {stdout}");
}

#[test]
fn interfaces_demo_interpret() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/interfaces_demo.rpy");
    let stdout = run_interpret(&path);
    assert!(
        !stdout.trim().is_empty(),
        "interfaces_demo should produce output"
    );
}

#[test]
fn classes_demo_interpret() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/classes_demo.rpy");
    let stdout = run_interpret(&path);
    assert!(
        stdout.contains("hello from Greeter"),
        "expected class greeting, got: {stdout}"
    );
}
