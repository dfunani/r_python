# How the fudge we build a programming language

**Audience:** you (and future contributors) who have written plenty of *application* code but have never shipped a *compiler*.  
**Promise:** no magic—just a pipeline of representations, each one slightly closer to the CPU than the last.

This doc is **meta**: it explains *how languages are built in general*, mapped to how **rPython** intends to do it. For product goals, read [DESIGN_SPEC.md](./DESIGN_SPEC.md).

## 1. The big picture: a factory line

Think of a compiler as a **series of translations**:

```text
Text  →  Tokens  →  Tree  →  Decorated tree  →  IR  →  IR  →  Machine code
        (lex)     (parse)   (analysis)        (high) (low)
```

Each stage:

- Has **inputs and outputs** with clear types (in Rust: structs/enums).
- Should be **testable on its own** (fixtures in, structured result out).
- Should **preserve source locations** (“spans”) as long as you still report errors to humans.

If any stage feels too hard, **split it** (e.g. “typecheck” becomes “name resolution” + “infer locals” + “check calls”).

## 2. Stage 0 — What is the language *for*?

Before code, answer in one page:

- **Execution model:** interpreted bytecode vs native binary vs JIT.
- **Memory model:** GC vs ownership vs hybrid.
- **Type system:** static vs gradual; generics; subtyping.
- **Interop:** C ABI? Rust crates? None at first?

rPython’s answers are in the design spec: **static**, **memory-safe**, **native via Rust codegen**, **not CPython-hosted**.

## 3. Stage 1 — Lexing (scanning)

**Job:** turn a Unicode string into a stream of **tokens** (`if`, `def`, `123`, `+`, `Newline`, `Indent`, `Dedent`, …).

**Why it exists:** parsing is easier on a small vocabulary than on raw characters.

**Practical tips**

- Treat **indentation** explicitly: Python-shaped languages usually emit `Indent`/`Dedent` tokens rather than brace soup.
- Keep **error messages local**: “unexpected character U+2028” beats a parser meltdown 200 lines later.

**Milestone smoke test:** `rpythonc --emit tokens` on a few files.

## 4. Stage 2 — Parsing

**Job:** turn tokens into an **Abstract Syntax Tree (AST)** that respects grammar precedence and associativity.

**Two common approaches**

1. **Hand-written recursive descent** — great control, great errors, more typing.
2. **Parser generator / combinators** — faster to iterate; still need careful error recovery.

**Milestone smoke test:** parse fixtures to AST; **pretty-print** back to text (doesn’t need to match byte-for-byte, but should be logically equivalent for tiny programs).

## 5. Stage 3 — Name resolution (“Who is `x`?”)

**Job:** build **scopes**: module, function, block; bind identifiers to **defs**; resolve **imports**; detect duplicate defs; maybe reject illegal assignments.

**Outputs**

- A **symbol table**: `SymbolId → definition site + type info placeholder`.
- Possibly an **annotated AST** (or parallel side maps) so later passes don’t redo lookup.

**Why before typechecking:** you can’t type a call to `foo` if you don’t know *which* `foo`.

## 6. Stage 4 — Typechecking (“Is this program nonsense?”)

**Job:** assign types to expressions; check calls; implement generics; enforce ownership/borrowing if that’s your model.

**Internals (typical)**

- **Unification:** unknown type variables solved by equations (`α = int`, `α = β`).
- **Trait / protocol solving:** constraint generation, sometimes orbit-scary—start tiny.
- **Coercions:** explicit only at first (casts), fewer surprises.

**Milestone smoke test:** 50 tiny programs with expected errors; snapshot the diagnostic text.

## 7. Stage 5 — High-level IR (HIR)

**Job:** simplify the AST into a smaller set of constructs: **desugar** syntax sugar, normalize loops, make implicit things explicit (e.g. drops, copies, ref takes—depends on model).

HIR is still **close to the language** but easier to optimize and lower.

## 8. Stage 6 — Mid-level IR (MIR / CFG / SSA)

**Job:** represent control flow as a **graph of basic blocks**; use **SSA** (static single assignment) so each “value” is assigned once—this makes many analyses trivial.

You’ll implement or reuse ideas like:

- **Phi nodes** at merge points (`if` joins).
- **Drop flags** or explicit destruction if not GC.
- **Dominators** for some optimizations later.

**Pro tip:** write a **MIR interpreter** *before* LLVM. If MIR runs wrong, LLVM will only amplify the pain.

## 9. Stage 7 — Codegen (LLVM, Cranelift, …)

**Job:** lower MIR to **LLVM IR** (or another backend), run passes, emit object files, link a binary.

**Why LLVM is popular:** industrial optimizer + many targets.

**Watchouts**

- **Debug info** (DWARF) mapping MIR/source spans—doable, not day one.
- **ABI** rules for calls: who cleans the stack, how structs are passed.

## 10. Stage 8 — Runtime (even for “systems” languages)

**Job:** tiny bits of **runtime** still exist: stack overflow hooks, panic/abort formatting, maybe allocation (if GC or arena), unwinding strategy.

For rPython’s Rust-aligned path, the “runtime” starts **small** and grows with the stdlib.

## 11. Stage 9 — Tooling humans actually touch

Rough priority order for happiness:

1. **Good errors** (spans, suggestions, stable codes).
2. **Test runner** for the language’s own tests.
3. **Formatter** (optional early).
4. **LSP** (later; needs incremental compilation story).

## 12. How you eat this elephant (order of work)

1. **Arithmetic expressions** end-to-end: lex → parse → eval (even in Rust without MIR) to learn spans and tests.
2. Add **control flow** + **functions** + a **symbol table**.
3. Add **types** for a micro-language subset.
4. Introduce **MIR + interpreter**.
5. Swap interpreter backend for **LLVM** on that same subset.
6. Only then grow syntax sugar and stdlib.

## 13. Recurring engineering truths (the fudge is logistics)

- **Semantics live in tests** — if it isn’t tested, the “spec” is whatever the compiler happened to do on Tuesday.
- **Diagnostics are a feature** — budget time like you would for a UI.
- **Incrementalism beats heroics** — a compiler that only supports `i32` math but *ships* beats a perfect type system in your head.

## 14. Further reading (external)

These are classic orientation materials (not endorsements of every design choice for rPython):

- *Engineering a Compiler* (Cooper & Torczon) — broad textbook coverage.
- LLVM’s **Language Reference** — once you emit IR, you’ll live here.
- Rust’s own **rustc dev guide** (conceptual parallels: HIR, MIR, borrow checker).

---

**Bottom line:** a programming language is mostly **a disciplined sequence of trees and graphs**, each with tests, until the last graph becomes bits the CPU understands. You’ve got this—one milestone at a time.
