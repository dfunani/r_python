# Installing rPython

You need **`rpythonc`** (the compiler) and a **C toolchain** (`cc` / Xcode CLT / build-essential) to produce native binaries from `.rpy` sources.

---

## Option 1 — Download a release binary (recommended)

Prebuilt `rpythonc` is published on [GitHub Releases](https://github.com/dfunani/r_python/releases) for:

| Target | Platform |
|--------|----------|
| `x86_64-unknown-linux-gnu` | Linux x64 |
| `aarch64-apple-darwin` | macOS Apple Silicon |
| `x86_64-apple-darwin` | macOS Intel |

### Quick install (macOS / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/dfunani/r_python/main/scripts/install.sh | bash
```

Or with an explicit version:

```bash
RPYTHON_VERSION=1.0.0 curl -fsSL https://raw.githubusercontent.com/dfunani/r_python/main/scripts/install.sh | bash
```

Ensure `~/.local/bin` is on your `PATH`.

### Manual download

1. Open [Releases](https://github.com/dfunani/r_python/releases).
2. Download `rpythonc-<target>.tar.gz` for your machine.
3. Extract and move `rpythonc` into a directory on your `PATH`:

```bash
tar -xzf rpythonc-aarch64-apple-darwin.tar.gz
chmod +x rpythonc-aarch64-apple-darwin/rpythonc
mv rpythonc-aarch64-apple-darwin/rpythonc ~/.local/bin/rpythonc
```

Verify:

```bash
rpythonc --version
```

---

## Option 2 — Build from source

Requires **Rust 1.78+** and a C compiler.

```bash
git clone https://github.com/dfunani/r_python.git
cd r_python
cargo build -p rpython_cli --release
```

Binary: `target/release/rpythonc`

```bash
export PATH="$PWD/target/release:$PATH"
rpythonc --run examples/hello.rpy
```

---

## Using the compiler

```bash
# Run in the MIR interpreter (no native code)
rpythonc --run program.rpy

# Compile to a native executable (requires cc on PATH)
rpythonc -o ./myapp program.rpy
./myapp

# Debug compiler stages
rpythonc --emit tokens program.rpy
rpythonc --emit ast program.rpy
rpythonc --emit hir program.rpy
rpythonc --emit mir program.rpy
```

---

## Requirements for native output (`-o`)

- **macOS:** Xcode Command Line Tools (`xcode-select --install`)
- **Linux:** `build-essential` or `gcc` + `libc-dev`
- **Windows:** not yet supported in release matrix (build from source with MSVC)

The compiler emits C and invokes your system linker automatically.

---

## Publishing a new release (maintainers)

```bash
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions (`.github/workflows/release.yml`) builds matrix artifacts and attaches them to the GitHub Release.
