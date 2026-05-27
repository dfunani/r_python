# Implementation status (P0–P12)

**Last updated:** 2026-05-27  
**Authoritative spec:** [IMPLEMENTATION.md](./IMPLEMENTATION.md) (target architecture)  
**This document:** what is **actually built** vs spec acceptance criteria.

Use this file to plan work, review PRs, and avoid assuming a feature works because the AST or parser accepts it.

---

## Executive summary

| Milestone band | Phases | Overall |
|----------------|--------|---------|
| Foundations | P0–P1 | **Done** |
| Front-end | P2–P4 | **Partial** (parse/typeck exist; fixture suites thin) |
| Middle-end | P5–P6 | **Partial** (HIR/MIR/interpreter + **C** backend, not LLVM) |
| Language features | P7–P11 | **Stub / partial** (AST/parser ahead of codegen & tests) |
| Tooling | P12 | **Not started** (basic CI only) |

**End-to-end today:** single-file `.rpy` → lex → parse → resolve → typecheck → HIR → MIR → borrowck (no-op) → C codegen → link → native binary or MIR `--run`.

**Largest spec gaps:** LLVM/inkwell backend, real borrowck, `stdlib/`, `tests/{parser,typeck,mir,ui,programs}`, `docs/LANGUAGE.md` (now started), multi-file crates, trait/mono codegen.

---

## Status legend

| Label | Meaning |
|-------|---------|
| **done** | Acceptance criteria met for this repo’s current scope |
| **partial** | Substantial code; acceptance criteria not met |
| **stub** | Types/API present; behavior missing or pass-through |
| **not started** | No meaningful implementation |

---

## Phase matrix

| Phase | Name | Status | Acceptance (spec §19) | Notes |
|-------|------|--------|------------------------|-------|
| **P0** | Workspace bootstrap | **done** | `cargo build` workspace; CI; `examples/hello.rpy` | 18 crates; `tools/*` not in workspace |
| **P1** | Lexer (M0) | **done** | `--emit tokens`; INDENT/DEDENT; invalid char spans | `tests/lexer/` (2 fixtures) |
| **P2** | Parser + AST (M1) | **partial** | AST snapshots; `tests/parser/*` round-trip | `crates/rpython_parse/tests/return_value.rs` only; no JSON snapshots |
| **P3** | Name resolution (M2) | **partial** | Multi-file crate; import graph tests | Single-file `resolve_crate` only |
| **P4** | Typechecker (M3) | **partial** | 50+ `tests/typeck`; `docs/LANGUAGE.md` | Micro subset: `int`/`bool`/`str`, `def`, `if`/`while`, `return`, calls |
| **P5** | HIR + MIR + interp (M4) | **partial** | `tests/mir/*`; HIR/MIR snapshots | Interpreter + `print` builtin; `--emit hir` unimplemented |
| **P6** | Codegen + runtime (M5) | **partial** | LLVM binary; no panic on bad input | **C backend** via `cc`, not inkwell; `hello.rpy` works |
| **P7** | Structs, enums, impls (M6) | **partial** | Struct/enum/match/impl e2e | Parsed + partial typeck; MIR aggregate lowering stubbed |
| **P8** | Traits + mono | **stub** | `traits_demo.rpy` compiles | `ImplTable` skeleton; no static dispatch proof |
| **P9** | Borrowck + drops | **stub** | Move/`&mut` errors | `borrowck_crate` is identity |
| **P10** | Stdlib + test runner (M7) | **stub** | `rpython test`; stdlib tests | `rpython_test_runner` library only; no `stdlib/` |
| **P11** | Surface completion | **partial** | `LANGUAGE.md` matches compiler; `tests/ui/` | Parser knows many keywords; few UI tests |
| **P12** | Tooling hardening | **not started** | macOS CI; release workflow; cache, `-g`, benches | Linux CI: fmt/clippy/test only |

---

## Crate map (current)

| Crate | Role | Maturity |
|-------|------|----------|
| `rpython_span` | Spans, `SourceMap` | done |
| `rpython_errors` | Diagnostics, stable codes | partial (no per-code markdown gen) |
| `rpython_ids` | `DefId`, `LocalId`, … | done |
| `rpython_syntax` | Lexer, tokens | done |
| `rpython_ast` | AST + arena + visitor | partial |
| `rpython_parse` | Recursive-descent parser | partial |
| `rpython_resolve` | Scopes, `DefMap`, builtins | partial (single file) |
| `rpython_types` | `TyKind`, layout hooks | partial |
| `rpython_typeck` | Inference, checks, trait stubs | partial |
| `rpython_hir` | Typed IR nodes | partial |
| `rpython_hir_build` | AST → HIR | partial |
| `rpython_mir` | MIR + pretty + **interpreter** | partial |
| `rpython_mir_build` | HIR → MIR | partial |
| `rpython_borrowck` | Loans, moves, drops | **stub** |
| `rpython_codegen_llvm` | **Misnamed:** emits C | partial (should gain LLVM behind feature) |
| `rpython_runtime` | C runtime (`rt.c`) | partial |
| `rpython_driver` | Pipeline, link | partial |
| `rpython_cli` | `rpythonc` | partial |
| `rpython_test_runner` | `#[test]` via MIR | stub |

---

## Spec layout: present vs missing

Paths from [IMPLEMENTATION.md §2](./IMPLEMENTATION.md#2-repository-layout-full-tree):

| Path | Status |
|------|--------|
| `crates/*` (all 18) | present |
| `examples/hello.rpy` | present, **compiles & runs** |
| `examples/gcd.rpy` | present (see file; may need later phases) |
| `examples/traits_demo.rpy` | present (documented **P8** target) |
| `stdlib/` | scaffolded — see [stdlib/README.md](../stdlib/README.md) |
| `docs/LANGUAGE.md` | present — **implemented subset** |
| `docs/errors/` | present — manual pages per code |
| `tools/` | README only (generators not wired) |
| `tests/lexer/` | present |
| `tests/parser/` | README + roadmap (in-crate tests exist) |
| `tests/typeck/` | README placeholder |
| `tests/mir/` | README placeholder |
| `tests/ui/` | README placeholder |
| `tests/programs/` | e2e via workspace `tests/programs_e2e.rs` |
| `.github/workflows/ci.yml` | present (Linux) |
| `.github/workflows/release.yml` | present (manual dispatch stub) |
| `lib/` (artifact cache) | gitignored when used |

---

## CLI (`rpythonc`) today

| Flag / mode | Status |
|-------------|--------|
| `--emit tokens` | works |
| `--emit ast` | works |
| `--emit mir` | works |
| `--emit hir` | **not implemented** |
| `--emit llvm` | **not implemented** (C backend used for binaries) |
| `-o path` | works (C + system linker) |
| `--run` | works (MIR interpreter) |
| `--opt 0..3` | passed to C compiler |
| `--explain E####` | works (subset of codes) |
| `rpython test` | **not implemented** (library: `rpython_test_runner`) |

---

## Recommended implementation order (to reach P12)

1. **P2–P4 tests** — `tests/parser/`, `tests/typeck/` snapshots; expand LANGUAGE.md as rules land.
2. **P5** — `tests/mir/`; implement `--emit hir`; fix aggregate/ref lowering stubs in `rpython_mir_build`.
3. **P6** — inkwell LLVM backend behind `feature = "llvm"`; keep C backend for fast bootstrap CI.
4. **P7–P8** — enum/struct MIR + codegen; trait dispatch + `traits_demo.rpy` green.
5. **P9** — real `rpython_borrowck`; emit `Drop` terminators.
6. **P10** — `stdlib/core`; wire `rpythonc test`.
7. **P11** — modules, `for`/`match` e2e, `tests/ui/`.
8. **P12** — incremental cache, DWARF, macOS CI, release binaries, benches.

---

## How to update this document

When closing a phase or acceptance item:

1. Change the phase row in **Phase matrix**.
2. Update **Spec layout** if new directories land.
3. Extend [LANGUAGE.md](./LANGUAGE.md) for user-visible behavior.
4. Add or link error docs under [errors/](./errors/).

Do **not** mark P12 **done** until release workflow ships installable `rpythonc` artifacts on Linux and macOS per spec.
