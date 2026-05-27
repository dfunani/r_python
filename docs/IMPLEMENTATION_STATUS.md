# Implementation status (P0–P12)

**Last updated:** 2026-05-27  
**Release:** **v2.0.0**  
**Authoritative spec:** [IMPLEMENTATION.md](./IMPLEMENTATION.md)  
**This document:** what is **actually built** vs spec acceptance criteria.

---

## Executive summary

| Milestone band | Phases | Overall |
|----------------|--------|---------|
| Foundations | P0–P1 | **Done** |
| Front-end | P2–P4 | **Partial** (parser/typeck; fixture suites growing) |
| Middle-end | P5–P6 | **Partial** (HIR/MIR/interpreter + **C** backend) |
| Language features | P7–P11 | **Partial** (structs, classes, interfaces, methods, while, `%`) |
| Tooling | P12 | **Partial** (CI, releases, install script) |

**End-to-end today:** single-file `.rpy` → lex → parse → resolve → typecheck → HIR → MIR → borrowck → C codegen → link → native binary, or `rpythonc run` (MIR interpreter).

**v2 examples that run:** `hello.rpy`, `gcd.rpy`, `interfaces_demo.rpy`, `classes_demo.rpy`, `traits_demo.rpy`.

**Remaining spec gaps:** LLVM/inkwell backend, multi-file modules, exhaustive match/for, full stdlib linkage, 50+ typeck/MIR/UI tests, DWARF/incremental cache.

---

## Phase matrix (v2.0)

| Phase | Name | Status | Notes |
|-------|------|--------|-------|
| **P0** | Workspace bootstrap | **done** | 18 crates, CI, releases |
| **P1** | Lexer | **done** | INDENT/DEDENT, `tests/lexer/` |
| **P2** | Parser + AST | **partial** | Full item surface; snapshot suite planned |
| **P3** | Name resolution | **partial** | Single-file; locals/params/impl methods |
| **P4** | Typechecker | **partial** | `int`/`bool`/`str`, calls, methods, structs, `%` |
| **P5** | HIR + MIR + interp | **partial** | While loops, aggregates, fields, calls |
| **P6** | Codegen + runtime | **partial** | C backend; `hello` native + interpret |
| **P7** | Structs, enums, classes | **partial** | Struct/class/enum parse + struct/class e2e |
| **P8** | Interfaces + dispatch | **partial** | `interface`/`trait`, `impl`, static method dispatch |
| **P9** | Borrowck + drops | **partial** | Move tracking scaffold (not full Rust-style yet) |
| **P10** | Stdlib + test runner | **partial** | `stdlib/core`, `collections`; `rpythonc test` library |
| **P11** | Surface completion | **partial** | `LANGUAGE.md` updated; UI tests planned |
| **P12** | Tooling hardening | **partial** | Release tarballs, install script |

---

## CLI (`rpythonc`) v2

| Command / flag | Status |
|----------------|--------|
| `rpythonc run` | works (MIR interpreter) |
| `rpythonc build -o` | works (C + system linker) |
| `rpythonc test` | partial (library; stdlib tests planned) |
| `--emit tokens` / `ast` / `high-level-ir` / `mid-level-ir` | works |
| `--emit llvm` | not implemented (C backend) |
| `--explain E####` | works (subset) |

---

## Stdlib (v2 scaffold)

| Path | Status |
|------|--------|
| `stdlib/core/prelude.rpy` | present |
| `stdlib/core/option.rpy` | present (source; not yet wired to compiler) |
| `stdlib/collections/vec.rpy` | present (source) |

Builtins in the compiler: `print` (`int`/`bool`/`str`).

---

## How to update this document

When closing a phase: update the matrix, [LANGUAGE.md](./LANGUAGE.md), and [V2_ROADMAP.md](./V2_ROADMAP.md).
