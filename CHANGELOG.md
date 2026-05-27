# Changelog

## 1.0.0 — 2026-05-27

First public release of the rPython compiler workspace.

### Compiler

- Lexer through MIR pipeline with C native codegen and MIR interpreter
- `rpythonc` CLI: `--emit {tokens,ast,hir,mir}`, `-o`, `--run`, `--test`, `--explain`
- Examples: `hello.rpy` (end-to-end), `gcd.rpy`, `traits_demo.rpy` (future P8)

### Documentation

- Full target spec (`docs/IMPLEMENTATION.md`) and [P0–P12 status](docs/IMPLEMENTATION_STATUS.md)
- [Language reference](docs/LANGUAGE.md) for the implemented subset
- Official website: [r_python_web](https://github.com/dfunani/r_python_web)

### Known limitations (post-1.0 roadmap)

- LLVM backend (C backend used for native builds)
- Real borrow checker, stdlib sources, multi-file crates, traits codegen
