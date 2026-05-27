# Code naming conventions (v2)

Readable names for reviewers and new contributors. **Avoid opaque abbreviations** in public functions, CLI flags, and driver APIs.

## Intermediate representations

| Abbreviation | Write instead (Rust) | Meaning |
|--------------|----------------------|---------|
| HIR | `high_level_intermediate` / `HighLevelIntermediate` | Typed IR after typecheck |
| MIR | `mid_level_intermediate` / `MidLevelIntermediate` | SSA-style IR before codegen |
| AST | `abstract_syntax` / `AbstractSyntax` | Parse tree |

**Crate names** (`rpython_hir`, `rpython_mir`) stay for dependency stability; document them in crate `README` with spelled-out titles.

## Language surface (user-facing)

| Avoid | Prefer |
|-------|--------|
| `trait` | `interface` |
| defaulting to `struct` for OOP | `class` for behavior + state; `struct` only for plain data |

## Functions (examples)

| Avoid | Prefer |
|-------|--------|
| `emit_hir` | `emit_high_level_intermediate_representation` |
| `emit_mir` | `emit_mid_level_intermediate_representation` |
| `emit_ast` | `emit_abstract_syntax_tree` |
| `build_mir` | `build_mid_level_intermediate_representation` (re-export alias OK) |

## CLI

- Prefer spelled-out `--emit high-level-ir` with aliases `hir`, `mir`, `ast`.
- `rpythonc run file.rpy` — primary interpret path.
- `rpythonc build -o out file.rpy` — native compile path.
- `--help` / `-h` always available (via `clap`).

## When abbreviations are OK

- Local loop indices, `match` arms on well-known enum variants inside a single crate.
- Industry-standard terms in **private** LLVM APIs (`inkwell`).
