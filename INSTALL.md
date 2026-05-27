# Installing rPython

You need **`rpythonc`** (the compiler) and a **C toolchain** (`cc` / Xcode CLT / build-essential) to produce native binaries from `.rpy` sources.

---

## Option 1 — Download a release binary (recommended)

Prebuilt `rpythonc` is published on [GitHub Releases](https://github.com/dfunani/r_python/releases) for:

| Target | Platform |
|--------|----------|
| `x86_64-unknown-linux-gnu` | Linux x64 |
| `aarch64-unknown-linux-gnu` | Linux ARM64 (e.g. Raspberry Pi, ARM VMs) |
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

Replace `v1.0.0` with the [latest release](https://github.com/dfunani/r_python/releases/latest) tag if newer.

**Stable alias** (recommended — same as `/releases/latest/download/...`):

```bash
curl -fL -O https://github.com/dfunani/r_python/releases/latest/download/rpythonc-aarch64-apple-darwin.tar.gz
tar -xzf rpythonc-aarch64-apple-darwin.tar.gz
install -m 755 rpythonc ~/.local/bin/rpythonc
```

**Versioned filename** (from the release assets list):

```bash
curl -fL -O https://github.com/dfunani/r_python/releases/download/v1.0.0/rpythonc-1.0.0-x86_64-unknown-linux-gnu.tar.gz
tar -xzf rpythonc-1.0.0-x86_64-unknown-linux-gnu.tar.gz
install -m 755 rpythonc ~/.local/bin/rpythonc
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
rpythonc run program.rpy
# legacy: rpythonc --run program.rpy

# Compile to a native executable (requires cc on PATH)
rpythonc build -o ./myapp program.rpy
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

## Troubleshooting

### `bash: tmp: unbound variable` after install

Fixed in current `main` — re-run install:

```bash
curl -fsSL https://raw.githubusercontent.com/dfunani/r_python/main/scripts/install.sh | bash
```

Older scripts used a `local` temp dir; the cleanup trap ran after `main` exited and tripped `set -u`.

### `failed to read examples/hello.rpy`

The install script does not ship examples. Create a file in **your current directory** or clone the repo:

```bash
git clone https://github.com/dfunani/r_python.git
cd r_python
rpythonc run examples/hello.rpy
```

### `rpythonc` exits with `killed` (no message)

Usually the Linux OOM killer (exit 137) or a very old binary. Try:

1. **Use real rPython syntax** — renaming `hello.py` is not enough; Python syntax is not supported.
2. **Build from source** (recommended on ARM Linux for latest fixes):

```bash
sudo apt-get install -y build-essential curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
git clone https://github.com/dfunani/r_python.git
cd r_python
cargo build -p rpython_cli --release
./target/release/rpythonc run examples/hello.rpy
```

3. Check memory: `dmesg | tail` for `Out of memory` / `Killed process`.

### Still on v1.0.0 from `releases/latest`

GitHub **latest** points at the newest tag. After **v2.0.0** is published, re-run install or set `RPYTHON_VERSION=2.0.0`.

---

## Publishing a new release (maintainers)

```bash
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions (`.github/workflows/release.yml`) builds matrix artifacts and attaches them to the GitHub Release.
