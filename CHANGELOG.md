# Changelog

## 2.0.0 — 2026-05-27

### Language & compiler

- **`interface`** keyword (`trait` deprecated alias); **`class`** / **`struct`** with method dispatch
- **`impl Interface for Type`** static dispatch; struct literals and field access
- **`while`**, **`%`**, assignment to locals/parameters, multi-function programs (`gcd.rpy`)
- MIR: aggregates, field projections, loop CFG, improved rvalue lowering
- Name resolution: params in `def_map`, locals registered, impl methods on types
- Borrowck scaffold (move tracking; full diagnostics planned)
- **`stdlib/`** scaffold: `core/prelude.rpy`, `core/option.rpy`, `collections/vec.rpy`

### Language & DX

- CLI subcommands: `run`, `build`, `test`, `explain`, `tokens`; `-r` / `--run` retained
- Verbose emit stages: `high-level-ir`, `mid-level-ir` (aliases `hir`, `mir`)
- Verbose driver API names; `CompilationStage` replaces abbreviated `EmitStage`

### Examples (e2e via `rpythonc run`)

- `hello.rpy`, `gcd.rpy`, `interfaces_demo.rpy`, `classes_demo.rpy`, `traits_demo.rpy`

### Docs & web

- [r_python_web](https://github.com/dfunani/r_python_web) — v2.0 site copy, roadmap phases updated
- [V2_ROADMAP.md](docs/V2_ROADMAP.md), [NAMING.md](docs/NAMING.md), [LANGUAGE.md](docs/LANGUAGE.md)

### Known limitations (post-2.0)

- LLVM backend (C backend used for native builds)
- Full borrow-checker, automatic stdlib loading, multi-file crates
- `match` / `for` / exhaustive enums through codegen

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
