# rPython

**rPython** is a planned **memory-safe, statically typed** language in the **Python surface-family**: familiar indentation syntax, many of the same keywords and data-model ideas, but **not** hosted on **CPython**. Instead, programs are **compiled through a Rust toolchain** (front-end in Rust, middle-end in Rust, native codegen via LLVM or similar) so safety and performance properties come from Rust’s ecosystem, not from CPython’s object model or refcounting C API.

The compiler runs **single-file** programs end-to-end (parse → typecheck → MIR → C codegen or interpreter). See [Implementation status (P0–P12)](./IMPLEMENTATION_STATUS.md) and [Language reference (implemented subset)](./LANGUAGE.md).

```bash
cargo run -p rpython_cli -- --run examples/hello.rpy
```

## Documentation

- [Design specification](./DESIGN_SPEC.md) — language shape, type system, runtime model, compilation strategy.
- [Implementation plan](./IMPLEMENTATION_PLAN.md) — milestones, crates, bootstrap path, testing.
- [**How the fudge we build a programming language**](./HOW_THE_FUDGE_WE_BUILD_A_PROGRAMMING_LANGUAGE.md) — plain-language tour from tokens to machine code: what each compiler stage does, how to test it, and how to sequence the work so the project does not drown in ambition.

## Naming note (important)

The PyPy project uses the name **“RPython”** for a *restricted Python subset used to implement PyPy itself*. **This rPython is a different effort**: a new language design and compiler, not PyPy’s restricted language. If the name proves too confusing in search or package managers, we can rename (e.g. `rpy`, `safe-py`, `ferrous-python`) later.

## Goals (short)

- **Static types** everywhere that matters; no gradual “optional types” as the default mode (exact strictness TBD in the design spec).
- **Memory safety** by construction in the compiled artifact (no raw pointers in user code; defined aliasing rules).
- **No dependency on CPython** to *run* rPython programs—only optionally as a host for the **compiler** during development if we ever embed CPython for bootstrapping (not the default story).

## License

TBD.
