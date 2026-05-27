# rPython — Implementation plan

**Version:** 0.1 (planning)  
**Related:** [Design specification](./DESIGN_SPEC.md) · [How the fudge we build a programming language](./HOW_THE_FUDGE_WE_BUILD_A_PROGRAMMING_LANGUAGE.md)

## 0. Milestones (vertical slices)

| Milestone | You can do this at the end |
|-----------|----------------------------|
| **M0** | `rpythonc --emit tokens` on a toy file |
| **M1** | Parse to AST; pretty-print round-trip for tiny programs |
| **M2** | Name resolution + basic modules |
| **M3** | Typecheck a **micro** language: `int`, `bool`, functions, `if`, `while`, `return` |
| **M4** | Lower to HIR/MIR subset; interpret MIR in a debug interpreter (no LLVM yet) |
| **M5** | LLVM codegen for that subset; produce a **static** binary `hello` |
| **M6** | Structs + traits; method dispatch |
| **M7** | Standard library v0 + `rpython test` harness |

Each milestone should close with **docs + tests + one demo program**.

## 1. Proposed Rust workspace

```
rpython/
  Cargo.toml
  crates/
    rpython_syntax/        # lexer, tokenizer errors
    rpython_parse/         # grammar → AST
    rpython_ast/           # AST definitions, visitors, spans
    rpython_resolve/       # scopes, imports, symbol ids
    rpython_typeck/        # inference, trait solving (start tiny)
    rpython_hir/           # typed tree / simplified control
    rpython_mir/           # SSA-like IR, drops, cfg
    rpython_codegen_llvm/  # inkwell / llvm-sys (pick one)
    rpython_runtime/       # tiny runtime helpers (panics hooks, abort messages)
    rpython_cli/           # `rpythonc`, flags, driver pipeline
  tests/
    ui/                    # snapshot tests for errors (optional)
    programs/              # small end-to-end programs
```

## 2. Bootstrap strategy

1. **Self-hosting is not an early goal** — the compiler stays Rust.
2. **Golden tests** drive semantics before performance.
3. **MIR interpreter before LLVM** — validates lowering without codegen bugs dominating.

## 3. Grammar delivery

- Start with a **hand-written recursive descent** parser for transparency, or use **logos + chumsky**-style combinator lexer/parser—decide in M0 spike.
- Keep grammar **small** until M3 is green.

## 4. Testing strategy

| Layer | What to test |
|-------|----------------|
| Lex/parse | Snapshot invalid inputs; AST equality on valid |
| Typecheck | Unit tests per typing rule + regression fixtures |
| MIR | Execution equivalence vs reference interpreter |
| LLVM | Run produced binaries in CI (linux first) |

## 5. CI / quality gates

- `cargo fmt`, `clippy -D warnings`, `cargo test`.
- MSRV policy documented once chosen.
- Fuzz lexer/parser only in M1+ (oss-fuzz later).

## 6. Documentation deliverables (living)

- Language reference (start as `docs/LANGUAGE.md` when grammar stabilizes).
- RFC template for breaking changes.
- This implementation plan updated every milestone.

## 7. Immediate next actions

1. Pick **memory model** for v1 (ownership-first recommended) and lock into design spec.
2. Spike: parse + evaluate **arithmetic expressions** in Rust end-to-end (warmup).
3. Define **first 10 language features** list and freeze until M5 ships.

## 8. Definition of Done (per milestone)

- Demo program checked into `examples/`.
- Compiler emits actionable errors (no panics on invalid input).
- README “try it” section updated.
