# Changelog

## 2.0.0 — 2026-05-27

### Language & DX

- **`interface`** keyword (replaces user-facing “trait”; `trait` deprecated alias)
- **`class`** as default OOP; **`struct`** for plain data only (documented)
- CLI subcommands: `run`, `build`, `test`, `explain`, `tokens`; `-r` / `--run` retained
- Verbose emit stages: `high-level-ir`, `mid-level-ir` (aliases `hir`, `mir`)
- Verbose driver API names; `CompilationStage` replaces abbreviated `EmitStage`

### Docs & web

- [r_python_web](https://github.com/dfunani/r_python_web) — official docs, install, playground
- [V2_ROADMAP.md](docs/V2_ROADMAP.md), [NAMING.md](docs/NAMING.md)

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
