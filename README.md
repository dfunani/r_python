# rPython

**rPython** is a memory-safe, statically typed language with Python-shaped syntax, compiled to native code via a Rust toolchain (not CPython).

## Try it

Requires Rust **1.78+**. Native binaries use a **C backend** today (LLVM is planned per the full spec).

```bash
cargo build -p rpython_cli

# Lex / AST / MIR
cargo run -p rpython_cli -- --emit tokens examples/hello.rpy
cargo run -p rpython_cli -- --emit ast examples/hello.rpy

# Interpret or compile
cargo run -p rpython_cli -- --run examples/hello.rpy
cargo run -p rpython_cli -- -o ./hello examples/hello.rpy && ./hello
```

## Documentation

- [Design specification](docs/DESIGN_SPEC.md)
- [Implementation plan](docs/IMPLEMENTATION_PLAN.md)
- [Full implementation spec](docs/IMPLEMENTATION.md)
- [**P0–P12 implementation status**](docs/IMPLEMENTATION_STATUS.md)
- [Language reference (implemented subset)](docs/LANGUAGE.md)
- [How we build a programming language](docs/HOW_THE_FUDGE_WE_BUILD_A_PROGRAMMING_LANGUAGE.md)

## Naming note

PyPy’s **RPython** is a restricted Python subset for bootstrapping PyPy. **This project is unrelated** — a new language and compiler.

## License

MIT OR Apache-2.0 — see LICENSE.
