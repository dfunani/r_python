# rPython v2.0 roadmap

**Goal:** Complete the [P0–P12](./IMPLEMENTATION.md#19-phased-delivery-map) specification as a viable, documented language — not only a micro-subset.

**v1.0 shipped:** single-file compile/run, C backend, installable `rpythonc`, basic CI/releases.  
**v2.0 target:** full surface language, LLVM or production C backend, real borrow checking, stdlib, modules, and the official website.

---

## Language design (v2 locked)

| v1 / Rust term | v2 user-facing term | Notes |
|----------------|---------------------|--------|
| trait | **interface** | Like Java/C# interfaces; keyword `interface` |
| trait (deprecated) | `trait` | Accepted with warning until v3 |
| struct | **struct** (data-only) | POD, C layout, no methods — use sparingly |
| class | **class** (default OOP) | Methods, inheritance, primary user type |
| `--run` | **`run` subcommand** or `-r` / `--run` | Interpreter path for development |
| `hir` / `mir` (CLI) | **`high-level-ir` / `mid-level-ir`** | Short forms kept as aliases |

See [NAMING.md](./NAMING.md) for Rust codebase naming (no opaque abbreviations in public APIs).

---

## Phase completion tracker (v2)

| Phase | v2 deliverable | Status |
|-------|----------------|--------|
| P0 | Workspace, releases, website repo | **done** |
| P1 | Lexer + fixtures | **done** |
| P2 | Parser snapshots, recovery | in progress |
| P3 | Multi-file crates, imports | planned |
| P4 | 50+ typeck tests, full `LANGUAGE.md` | planned |
| P5 | MIR tests, verbose IR dumps, SSA | planned |
| P6 | LLVM feature flag + C backend | planned |
| P7 | Class + enum + interface codegen | planned |
| P8 | Interface dispatch + monomorphization | planned |
| P9 | Real borrowck + drops | planned |
| P10 | `stdlib/`, `rpythonc test` | planned |
| P11 | `match`, `for`, modules, UI tests | planned |
| P12 | DWARF, cache, benches, `cargo install` | planned |

---

## v2.0 release criteria

1. **Website** — [r_python_web](https://github.com/dfunani/r_python_web): install, tutorials, examples, playground UX.
2. **Compiler** — `rpythonc run`, `rpythonc build -o`, `rpythonc test`, `--help` documents compile vs interpret.
3. **Examples** — `hello`, `gcd`, `interfaces_demo`, `classes_demo` all run on release binaries.
4. **Docs** — `LANGUAGE.md` matches compiler; interfaces and classes explained.
5. **CI** — Linux + macOS; release tarballs per platform.

---

## Implementation order (sprints)

### Sprint A — Terminology & DX (current)
- `interface` keyword; `trait` deprecated alias
- Verbose pipeline function names; CLI `high-level-ir` / `mid-level-ir`
- `rpythonc` subcommands: `run`, `build`, `test`, `explain`
- `docs/NAMING.md`, this roadmap, website scaffold

### Sprint B — Front-end depth
- `tests/parser/`, `tests/typeck/` suites
- Multi-file `rpython.toml` package root

### Sprint C — Middle-end
- Complete HIR/MIR lowering (aggregates, refs, calls)
- MIR interpreter parity with native codegen

### Sprint D — Back-end & safety
- LLVM behind `feature = "llvm"`; keep C for bootstrap
- `rpython_borrowck` loans/moves/drops

### Sprint E — Surface + stdlib
- Classes, interfaces, `match`, `for`, modules
- `stdlib/core` + `rpythonc test`

### Sprint F — P12 polish
- Debug symbols, incremental cache, benchmarks, book on website
