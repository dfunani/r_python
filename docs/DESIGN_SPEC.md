# rPython — Design specification

**Version:** 0.1 (planning)  
**Status:** Draft

## 1. Problem statement

Teams want **Python-shaped** productivity and readability, but also:

- **Provable static types** (catch errors before run, enable IDE features).
- **Memory safety** without the CPython C-API footguns and without a large runtime interpreter in the hot path.
- **Native performance** for numerical / systems-style workloads where CPython + C extensions is a fragile split brain.

**rPython** targets that niche by being a **new language** with a Python-like syntax and semantics *where they align with static safety*, compiled **natively via Rust’s compiler infrastructure**, not executed by **CPython**.

## 2. Non-goals (initial)

- **Bug-for-bug CPython compatibility** — we inherit ideas, not semantics wholesale.
- **Dropping into CPython C API** from user code — excluded by design early on.
- **Full stdlib parity** with CPython on day one — we ship a **minimal, curated** standard library surface and grow deliberately.

## 3. Design principles

1. **Static by default** — types are required (inference can fill obvious locals, but public APIs are explicit).
2. **Memory safety without GC dogma** — default story is **Rust-like ownership + borrowing** *or* a **safe GC** for shared graphs; pick one primary model for v1 (see §6).
3. **Familiar surface** — significant whitespace, `def`, `class`, `if`/`elif`/`else`, comprehensions where typeable.
4. **Compiler is Rust** — lexer/parser/semantic/IR/codegen live in Rust; the runtime is Rust (statically linked) or small native stubs.
5. **Interop is explicit** — calling C or Rust has `unsafe`-style boundaries at the language level (keywords/attributes), never accidental.

## 4. Relationship to Python

| Python (CPython) idea | rPython stance (initial) |
|------------------------|---------------------------|
| Duck typing everywhere | Rejected as default; traits/protocols instead. |
| Mutable default arguments | Disallowed or desugared to explicit patterns. |
| `**kwargs` open extensibility | Restricted; requires typed mapping or builder APIs. |
| Dynamic `getattr` | Limited; reflection behind a capability boundary. |
| GIL + refcount object model | Not used; rPython has its own object layout / ownership rules. |

## 5. Syntax (high-level sketch)

*Illustrative only — grammar TBD in implementation milestones.*

```text
def gcd(a: int, b: int) -> int:
    while b != 0:
        a, b = b, a % b
    return a
```

- **Type annotations** on parameters and returns are part of the language, not optional comments.
- **Indentation** defines blocks; line joining rules similar to Python.
- **Deliberate cuts**: metaclasses, frame introspection, `eval` on arbitrary strings, `ctypes` footguns — absent or heavily gated.

## 6. Memory and concurrency model (decision required)

**Option A — Ownership + borrowing (Rust-aligned)**

- Pros: strongest safety story, no GC pauses, clearest “not CPython” story.
- Cons: learning curve; lifetime errors at compile time.

**Option B — Tracing GC + linear types for resources**

- Pros: more Python-like aliasing for graphs.
- Cons: harder performance predictability; still need escape analysis.

**Recommendation for planning:** prototype **Option A** for core language subset; keep IR lowering close enough to **MIR-like** concepts that we can reuse Rust’s backend expertise. Revisit GC for “Pythonic” object graphs in a later tier.

## 7. Type system (sketch)

- Nominal classes + **traits** (Rust-like) for ad-hoc polymorphism.
- Generics with declared variance rules (conservative defaults).
- **Sum types** (`enum`) first-class for errors and state machines.
- **Modules** as units of compilation; visibility (`pub`) explicit.
- **Effect system** deferred; start with simple `async` desugaring later.

## 8. Compilation architecture

```
Source (.rpy / .pyr — TBD)
  → Lex + Parse → CST/AST (spans preserved)
  → Name resolution + imports
  → Typecheck + trait solving
  → High-level IR (HIR)
  → Mid-level IR (MIR-like: SSA, control flow, drops)
  → LLVM IR (via inkwell) **or** Cranelift for faster compile/debug
  → Native binary + debuginfo (DWARF)
```

**Why Rust in the loop:** one toolchain for parser performance, safe compiler data structures, and mature codegen backends—without implementing a GC + JIT + C-API like CPython.

## 9. Standard library (v0 direction)

- `core`: integers, booleans, tuples, fixed arrays, strings (UTF-8), options/results.
- `io`: buffered files, paths (no implicit encoding guessing surprises—documented).
- `collections`: vec, map with chosen hash algorithm; **no** arbitrary `dict` of `Any`.
- **No** `importlib` magic at first.

## 10. Interop

- **`extern` blocks** for C ABI with unsafe boundary (exact syntax TBD).
- **Rust crate embedding** story deferred; start with static libs + C ABI.

## 11. Diagnostics and UX

- Stable error codes; rustc-style `--explain E1234` equivalent.
- JSON diagnostics for editors.
- Formatter / LSP later; not v0.

## 12. Risks

| Risk | Mitigation |
|------|------------|
| “Python but not Python” confusion | Clear positioning; examples; spec tests. |
| PyPy RPython name collision | Rename if needed; prominent README note. |
| Type system complexity explosion | Ship tiny core; grow via RFCs. |

## 13. Open questions

1. File extension and package layout (`pyproject.toml` compatibility? probably not initially).
2. Integer model: arbitrary precision vs fixed widths vs hybrid.
3. String type: `str` as `Utf8` only; bytes story.
4. Async: color vs real async; start synchronous only.
