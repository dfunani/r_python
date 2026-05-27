# rPython

**rPython** is a memory-safe, statically typed language with Python-shaped syntax, compiled to native code via a Rust toolchain (not CPython).

## Install

**Prebuilt binaries** (no Rust required): see [INSTALL.md](INSTALL.md) or [GitHub Releases](https://github.com/dfunani/r_python/releases).

```bash
curl -fsSL https://raw.githubusercontent.com/dfunani/r_python/main/scripts/install.sh | bash
```

You also need a C compiler (`cc`) on your PATH to use `rpythonc -o` (Xcode CLT on macOS, `build-essential` on Linux).

**From source** (Rust 1.78+):

```bash
cargo build -p rpython_cli --release
export PATH="$PWD/target/release:$PATH"
```

## Try it

```bash
# Interpret (fastest — no linker)
rpythonc --run examples/hello.rpy

# Native executable
rpythonc -o ./hello examples/hello.rpy
./hello

# Inspect the compiler
rpythonc --emit tokens examples/hello.rpy
rpythonc --emit ast examples/hello.rpy
rpythonc --emit mir examples/hello.rpy
```

## Documentation

- [Install guide](INSTALL.md)
- [Design specification](docs/DESIGN_SPEC.md)
- [Language reference (implemented subset)](docs/LANGUAGE.md)
- [**P0–P12 implementation status**](docs/IMPLEMENTATION_STATUS.md)
- [Full implementation spec](docs/IMPLEMENTATION.md)
- [Official website](https://github.com/dfunani/r_python_web) (docs & playground)

## Naming note

PyPy’s **RPython** is a restricted Python subset for bootstrapping PyPy. **This project is unrelated** — a new language and compiler.

## License

MIT OR Apache-2.0 — see [LICENSE](LICENSE).
