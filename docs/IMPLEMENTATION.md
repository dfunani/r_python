# rPython — Full implementation specification

**Version:** 1.0 (implementation blueprint)  
**Status:** Authoritative build spec for codegen / copy-paste scaffolding  
**Audience:** Implementers, codegen tools, contributors  
**Related:** [DESIGN_SPEC.md](./DESIGN_SPEC.md) · [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) · [HOW_THE_FUDGE_WE_BUILD_A_PROGRAMMING_LANGUAGE.md](./HOW_THE_FUDGE_WE_BUILD_A_PROGRAMMING_LANGUAGE.md)

This document is **not** a milestone plan. It is the **complete target architecture** for the rPython language and compiler: repository layout, every major type and IR node, pass boundaries, runtime ABI, standard library surface, CLI, and phased delivery order. **No executable code appears here** — only structures, responsibilities, and pseudosignatures sufficient to generate or hand-write the codebase.

> **Implementation progress:** see [**IMPLEMENTATION_STATUS.md**](./IMPLEMENTATION_STATUS.md) for what is built today vs each phase’s acceptance criteria (P0–P12).

---

## Table of contents

1. [Locked product decisions](#1-locked-product-decisions)
2. [Repository layout (full tree)](#2-repository-layout-full-tree)
3. [Workspace and crate dependency graph](#3-workspace-and-crate-dependency-graph)
4. [Shared foundations (`rpython_span`, `rpython_errors`, `rpython_ids`)](#4-shared-foundations)
5. [Lexer and tokens (`rpython_syntax`)](#5-lexer-and-tokens)
6. [Parser and AST (`rpython_parse`, `rpython_ast`)](#6-parser-and-ast)
7. [Name resolution (`rpython_resolve`)](#7-name-resolution)
8. [Type system (`rpython_types`, `rpython_typeck`)](#8-type-system)
9. [HIR (`rpython_hir`)](#9-hir)
10. [MIR (`rpython_mir`)](#10-mir)
11. [Borrow checking and drop elaboration](#11-borrow-checking-and-drop-elaboration)
12. [LLVM codegen (`rpython_codegen_llvm`)](#12-llvm-codegen)
13. [Runtime (`rpython_runtime`)](#13-runtime)
14. [Driver and CLI (`rpython_cli`, `rpython_driver`)](#14-driver-and-cli)
15. [Standard library (rPython source)](#15-standard-library)
16. [Diagnostics and error codes](#16-diagnostics-and-error-codes)
17. [Package, module, and build model](#17-package-module-and-build-model)
18. [Test harness and fixtures](#18-test-harness-and-fixtures)
19. [Phased delivery map (P0–P12)](#19-phased-delivery-map)
20. [Appendix A — Token catalog](#appendix-a--token-catalog)
21. [Appendix B — AST node catalog](#appendix-b--ast-node-catalog)
22. [Appendix C — HIR node catalog](#appendix-c--hir-node-catalog)
23. [Appendix D — MIR instruction catalog](#appendix-d--mir-instruction-catalog)
24. [Appendix E — Type kind catalog](#appendix-e--type-kind-catalog)

---

## 1. Locked product decisions

These choices are **fixed for v1 implementation** unless an RFC explicitly revises them.

| Area | Decision |
|------|----------|
| **File extension** | `.rpy` for sources; package manifest `rpython.toml` |
| **Memory model** | Rust-aligned **ownership + borrowing**; moves at assignment; explicit `mut`; no tracing GC in v1 |
| **Integer model** | `int` is **i64** by default; `i8`…`i64`, `u8`…`u64` as explicit types; `BigInt` deferred to stdlib tier 2 |
| **Float model** | `f32`, `f64` |
| **String model** | `str` is **UTF-8 owned** (`String` layout at runtime); `bytes` is `u8` slice/vec |
| **Bool** | `bool` — two-valued, not truthy integers |
| **Nullability** | `Option[T]` only; no `None` as untyped sentinel |
| **Classes** | Nominal; single inheritance for classes; traits for polymorphism |
| **Generics** | Monomorphization at codegen (no vtable for generic functions unless trait object) |
| **Async** | **Out of v1 compiler**; synchronous only until P11 |
| **Backend** | **LLVM 18+** via `inkwell`; Cranelift feature flag optional later |
| **Host compiler** | Rust **1.78+** MSRV (document in root `README`) |
| **Self-hosting** | Not a goal before P12 |

---

## 2. Repository layout (full tree)

Every path below should exist by end of full implementation. Empty leaf files may be `mod.rs` re-exports only.

```text
rPython/
├── Cargo.toml                          # workspace root
├── rpython.toml                        # example package manifest (documented)
├── README.md
├── LICENSE
├── .github/
│   └── workflows/
│       ├── ci.yml                      # fmt, clippy, test, mir-interp, e2e binaries
│       └── release.yml                 # tagged releases
├── docs/
│   ├── README.md
│   ├── DESIGN_SPEC.md
│   ├── IMPLEMENTATION_PLAN.md
│   ├── IMPLEMENTATION.md               # this file
│   ├── HOW_THE_FUDGE_WE_BUILD_A_PROGRAMMING_LANGUAGE.md
│   ├── LANGUAGE.md                     # user-facing grammar (generated/maintained at P4)
│   └── errors/                         # E0001.md … per stable error code
├── crates/
│   ├── rpython_span/
│   ├── rpython_errors/
│   ├── rpython_ids/
│   ├── rpython_syntax/
│   ├── rpython_ast/
│   ├── rpython_parse/
│   ├── rpython_resolve/
│   ├── rpython_types/
│   ├── rpython_typeck/
│   ├── rpython_hir/
│   ├── rpython_hir_build/
│   ├── rpython_mir/
│   ├── rpython_mir_build/
│   ├── rpython_borrowck/
│   ├── rpython_codegen_llvm/
│   ├── rpython_runtime/
│   ├── rpython_driver/                 # pipeline orchestration (library)
│   ├── rpython_cli/                    # binary entry
│   └── rpython_test_runner/            # `rpython test` (library + thin bin)
├── stdlib/                             # rPython standard library sources
│   ├── core/
│   ├── collections/
│   ├── io/
│   └── prelude.rpy
├── lib/                                # precompiled rlib artifacts cache (gitignored)
├── examples/
│   ├── hello.rpy
│   ├── gcd.rpy
│   └── traits_demo.rpy
├── tests/
│   ├── lexer/                          # .rpy + .tokens.expect
│   ├── parser/                         # .rpy + .ast.json.expect
│   ├── typeck/                         # .rpy + .stderr.expect
│   ├── mir/                            # .mir.expect + run via interpreter
│   ├── ui/                             # compile-fail / run-pass
│   └── programs/                       # e2e binaries
└── tools/
    ├── gen_errors.rs                   # generates docs/errors from registry
    └── gen_ast_visitor.rs              # optional codegen for AST visitors
```

---

## 3. Workspace and crate dependency graph

### 3.1 Root `Cargo.toml` (workspace members)

**Members:** all crates under `crates/*` plus `tools/*` as `[[bin]]` only where needed.

**Workspace dependencies (centralized versions):**

- `logos` — lexer
- `smol_str` — identifier storage
- `indexmap` — deterministic maps
- `rustc-hash` — `FxHashMap` in hot paths
- `inkwell` — LLVM bindings (features: `llvm18-0`)
- `serde`, `serde_json` — test snapshots
- `thiserror`, `anyhow` — errors (`thiserror` in libraries; `anyhow` only in CLI/tools)

### 3.2 Dependency DAG (must not cycle)

```text
rpython_cli
  └── rpython_driver
        ├── rpython_codegen_llvm
        │     ├── rpython_mir (+ rpython_mir_build)
        │     ├── rpython_borrowck
        │     └── rpython_runtime (metadata only for symbols)
        ├── rpython_typeck
        │     ├── rpython_resolve
        │     ├── rpython_types
        │     └── rpython_hir_build → rpython_hir → rpython_ast
        ├── rpython_parse → rpython_ast, rpython_syntax
        └── rpython_errors, rpython_span, rpython_ids

rpython_test_runner → rpython_driver (test APIs)
```

**Rule:** `rpython_ast` never depends on `rpython_typeck`. IR crates never depend on `rpython_parse`.

---

## 4. Shared foundations

### 4.1 `crates/rpython_span/`

**Purpose:** Source locations carried through every IR.

| File | Contents |
|------|----------|
| `lib.rs` | Re-exports |
| `span.rs` | `Span`, `SpanData` |
| `source_map.rs` | `SourceMap`, `SourceFile`, `FileId`, `BytePos`, `LineCol` |
| `hygiene.rs` | `SyntaxContext` (reserved for macros; v1 stub) |

**Types:**

```text
struct Span {
  file_id: FileId
  start: BytePos      // inclusive, UTF-8 byte offset
  end: BytePos        // exclusive
  ctxt: SyntaxContext // default 0 in v1
}

struct SourceFile {
  name: PathBuf
  contents: String
  line_starts: Vec<BytePos>   // built at load
}

struct SourceMap {
  files: IndexMap<FileId, SourceFile>
}
```

**Functions:**

- `SourceMap::load_file(path) -> (FileId, &SourceFile)`
- `SourceMap::line_col(span) -> (usize line, usize col)`
- `Span::merge(a, b) -> Span`
- `Span::dummy() -> Span` for synthesized nodes

### 4.2 `crates/rpython_errors/`

**Purpose:** Diagnostic reporting independent of pass.

| File | Contents |
|------|----------|
| `lib.rs` | |
| `diagnostic.rs` | `Diagnostic`, `Level`, `Label`, `Suggestion` |
| `emitter.rs` | `Emitter` trait; `HumanEmitter`, `JsonEmitter` |
| `codes.rs` | `ErrorCode` enum + `explain()` lookup |
| `handler.rs` | `Handler` — collects, caps count, abort on fatal |

```text
enum Level { Error, Warning, Note, Help }

struct Label {
  span: Span
  message: String
  primary: bool
}

struct Diagnostic {
  level: Level
  code: Option<ErrorCode>
  message: String
  labels: Vec<Label>
  suggestions: Vec<Suggestion>
  children: Vec<Diagnostic>   // notes attached
}

struct Handler {
  diagnostics: Vec<Diagnostic>
  errors: usize
  warnings: usize
  max_errors: usize           // default 50
}
```

### 4.3 `crates/rpython_ids/`

**Purpose:** Newtype IDs for arena indexing; stable across a compilation session.

| Type | Meaning |
|------|---------|
| `LocalId` | MIR/local variable index within function |
| `BlockId` | MIR basic block |
| `SymbolId` | Resolved name in a scope |
| `DefId` | Definition site (function, struct, const, module) |
| `TypeId` | Index into `TypeDatabase` |
| `TraitId` | Trait definition |
| `ImplId` | `impl Trait for Type` |
| `CrateId` | Root crate being compiled |
| `ModuleId` | Submodule within crate |
| `ExprId` / `StmtId` / `ItemId` | AST arena indices |
| `HIRBodyId` | HIR body owner |
| `MirFuncId` | Monomorphized MIR function instance |

All IDs implement: `Copy`, `Clone`, `Eq`, `Hash`, `Debug`, `from_usize`, `index()`.

---

## 5. Lexer and tokens

### 5.1 Crate: `rpython_syntax`

| File | Responsibility |
|------|----------------|
| `lib.rs` | Public API |
| `lexer.rs` | `Lexer::next_token() -> Result<Option<SpannedToken>, LexError>` |
| `token.rs` | `Token` enum, `TokenKind` |
| `indent.rs` | INDENT/DEDENT stack; `IndentQueue` |
| `string.rs` | String/bytes/f-string lexing (f-strings P8+) |
| `comment.rs` | `#` line comments |
| `error.rs` | `LexError` variants |

### 5.2 Indentation algorithm

Maintain:

- `indent_stack: Vec<usize>` — column widths (tabs expanded: tab stop 8)
- On `Newline`: emit `Newline`; if next line non-blank, compare column → emit zero or more `Indent` or `Dedent` before first token of line
- At EOF: pop stack with `Dedent` until empty

### 5.3 `SpannedToken`

```text
struct SpannedToken {
  kind: TokenKind
  span: Span
}
```

See [Appendix A](#appendix-a--token-catalog) for full `TokenKind` list.

### 5.4 Public API

```text
fn tokenize(source_map: &SourceMap, file_id: FileId) -> TokenStream
struct TokenStream { iter }
impl Iterator for TokenStream -> SpannedToken
```

---

## 6. Parser and AST

### 6.1 Crate: `rpython_ast`

Arena-backed AST. **All nodes carry `Span`** (or span per field where useful).

| File | Contents |
|------|----------|
| `lib.rs` | |
| `arena.rs` | `Arena<T>`, alloc helpers |
| `ids.rs` | Re-export `ExprId`, etc. |
| `expr.rs` | `Expr`, `ExprKind` |
| `stmt.rs` | `Stmt`, `StmtKind` |
| `item.rs` | `Item`, `ItemKind` — top-level & nested items |
| `pat.rs` | `Pat`, `PatKind` |
| `ty.rs` | `Ty`, `TyKind` — syntax types |
| `path.rs` | `Path`, `PathSegment` |
| `attr.rs` | Attributes (`@inline`, `@test`, …) |
| `visit.rs` | `Visitor` trait + walk_* functions |
| `display.rs` | Pretty-print for tests (`AstPrinter`) |

### 6.2 Crate: `rpython_parse`

| File | Responsibility |
|------|----------------|
| `lib.rs` | `parse_file`, `parse_module` |
| `parser.rs` | Recursive descent `Parser` |
| `expr.rs` | Pratt/precedence for expressions |
| `stmt.rs` | Statement parsing |
| `item.rs` | `def`, `class`, `enum`, `impl`, `import` |
| `recovery.rs` | Error recovery (sync points: `Newline`, `Dedent`, `def`, `class`) |
| `grammar.md` | Maintainer-facing grammar (EBNF) |

### 6.3 `Module` root

```text
struct Module {
  items: Vec<ItemId>
  span: Span
}
```

### 6.4 Parser entrypoints

```text
fn parse_module(tokens: TokenStream, arena: &Arena, handler: &Handler) -> Option<Module>
fn parse_expr(tokens: ..., ...) -> Option<ExprId>   // for REPL later
```

See [Appendix B](#appendix-b--ast-node-catalog).

---

## 7. Name resolution

### 7.1 Crate: `rpython_resolve`

| File | Responsibility |
|------|----------------|
| `lib.rs` | `resolve_crate(module) -> Resolution` |
| `scope.rs` | `Scope`, `ScopeKind` (Root, Module, Function, Block) |
| `ribs.rs` | Scope stack, shadowing rules |
| `symbols.rs` | `Symbol`, `Binding`, `NameBinding` |
| `imports.rs` | `import`, `from … import`, module path resolution |
| `def_map.rs` | `DefId -> DefKind` |
| `collect.rs` | First pass: collect all defs without bodies |
| `resolve_expr.rs` | Second pass: resolve paths in expressions |

### 7.2 `DefKind`

```text
enum DefKind {
  Module(ModuleId)
  Function { parent: DefId, sig_span: Span }
  Struct { fields: Vec<FieldDef> }
  Enum { variants: Vec<VariantDef> }
  Trait { methods: Vec<MethodSig> }
  Impl { trait_ref: Option<DefId>, self_ty: TypeId }
  Const { ty: TypeId }
  TypeAlias { ty: TypeId }
  ExternFn { abi: Abi }
  ExternBlock
  Local { owner: DefId, index: u32 }
  Param { owner: DefId, index: u32 }
}
```

### 7.3 `Resolution` output

```text
struct Resolution {
  def_map: DefMap
  symbol_map: HashMap<ExprId, SymbolId>   // path expressions
  module_tree: ModuleTree
  imports: Vec<ImportRecord>
}
```

**Rules:**

- Duplicate defs in same scope → `E0201`
- Use before def (locals) → `E0202`
- Unresolved import → `E0203`

---

## 8. Type system

### 8.1 Crate: `rpython_types`

| File | Contents |
|------|----------|
| `lib.rs` | |
| `ty.rs` | `TyKind`, `TypeDatabase` |
| `subst.rs` | Substitution for generics |
| `fold.rs` | Type folder trait |
| `infer.rs` | Inference variable helpers |
| `trait_ref.rs` | `TraitRef { trait_id, args }` |
| `layout.rs` | `Layout`, `Align`, `Size` — for codegen & MIR |

### 8.2 `TyKind` (canonical)

See [Appendix E](#appendix-e--type-kind-catalog).

`TypeDatabase` stores:

```text
struct TypeDatabase {
  kinds: Vec<TyKind>
  interner: HashMap<TyKind, TypeId>
}
```

### 8.3 Crate: `rpython_typeck`

| File | Responsibility |
|------|----------------|
| `lib.rs` | `typecheck(resolution, module) -> TypedProgram` |
| `check_fn.rs` | Function body checking |
| `check_expr.rs` | Expression typing |
| `check_pat.rs` | Pattern exhaustiveness (P7+) |
| `unify.rs` | Unification engine |
| `constraints.rs` | Constraint generation |
| `traits.rs` | Trait obligation solving |
| `coercion.rs` | Allowed coercions (minimal: none except widen numerics optional) |
| `ownership.rs` | Mutability, move semantics at type level |
| `well_known.rs` | Built-in types and traits |

### 8.4 `TypedProgram`

```text
struct TypedProgram {
  types: TypeDatabase
  expr_types: HashMap<ExprId, TypeId>
  pat_types: HashMap<PatId, TypeId>
  item_sigs: HashMap<DefId, TypeId>
  impl_table: ImplTable
  obligations: Vec<FulfilledObligation>   // audit log
}
```

### 8.5 Trait solving (v1 algorithm)

1. Generate obligations at call sites and method references.
2. Search `impl_table` for matching `impl_id` (coherence: orphan rules enforced).
3. If ambiguous → `E0305`; if missing → `E0306`.
4. Monomorphization list: `MonoInstance { def_id, subst }`.

### 8.6 Built-in traits (minimum)

| Trait | Methods |
|-------|---------|
| `Eq` | `fn eq(self, other: Self) -> bool` |
| `Clone` | `fn clone(self) -> Self` |
| `Copy` | marker |
| `Drop` | `fn drop(self)` |
| `Iterator` | `fn next(self) -> Option[Item]` |
| `Add`, `Sub`, … | `fn add(self, rhs: Self) -> Self` per op |

---

## 9. HIR

HIR is **typed**, **desugared**, and **name-resolved**. One HIR item per `DefId` body.

### 9.1 Crate: `rpython_hir`

| File | Contents |
|------|----------|
| `lib.rs` | |
| `body.rs` | `Body`, `BasicBlock` (optional flat form) |
| `expr.rs` | `HirExpr`, `HirExprKind` |
| `stmt.rs` | `HirStmt` |
| `pat.rs` | `HirPat` |
| `owner.rs` | `HirOwner`, `HirOwnerKind` |
| `visit.rs` | HIR visitor |

### 9.2 Crate: `rpython_hir_build`

| File | Responsibility |
|------|----------------|
| `lib.rs` | `build_hir(typed_program, module) -> HirCrate` |
| `lower_expr.rs` | Desugar `and`/`or`, comprehensions (P8), `for` → loop+iter |
| `lower_item.rs` | Functions, impls |
| `place.rs` | `Place`, `PlaceKind` — lvalues |
| `rvalue.rs` | `Rvalue` — temporaries |

### 9.3 `HirCrate`

```text
struct HirCrate {
  owners: IndexMap<DefId, HirOwner>
}

enum HirOwnerKind {
  Function(HirBody)
  Static { ty: TypeId, val: HirExprId }
  Const { ... }
}
```

See [Appendix C](#appendix-c--hir-node-catalog).

**Desugarings (mandatory):**

- `a and b` → `if a { b } else { false }`
- `a or b` → `if a { true } else { b }`
- `for x in it:` → while-let over iterator
- Tuple assignment → destructuring
- Method call `x.foo(y)` → `Trait::foo(x, y)` with autoref rules

---

## 10. MIR

MIR is **SSA**, **control-flow graph**, **monomorphized**, with explicit **drops** and **moves**.

### 10.1 Crate: `rpython_mir`

| File | Contents |
|------|----------|
| `lib.rs` | |
| `body.rs` | `MirBody`, `BasicBlock`, `BasicBlockData` |
| `statement.rs` | `Statement`, `StatementKind` |
| `terminator.rs` | `Terminator`, `TerminatorKind` |
| `operand.rs` | `Operand`, `Constant` |
| `place.rs` | `Place`, `Projection` |
| `visit.rs` | MIR visitor |
| `pretty.rs` | `MirPrinter` for tests |

### 10.2 Crate: `rpython_mir_build`

| File | Responsibility |
|------|----------------|
| `lib.rs` | `build_mir(hir_crate, mono_instances) -> MirCrate` |
| `build_fn.rs` | Lower `HirBody` to CFG |
| `ssa.rs` | SSA construction, phi insertion |
| `aggregate.rs` | Struct/tuple/array aggregates |
| `switch.rs` | `match` lowering |

### 10.3 `MirBody` structure

```text
struct MirBody {
  args: Vec<Local>           // includes return place as local 0 convention
  locals: Vec<LocalDecl>     // type + mutability + span
  blocks: IndexVec<BlockId, BasicBlockData>
  source_scope: Vec<SourceScope>
}

struct LocalDecl {
  ty: TypeId
  mutability: Mutability
  span: Span
}

struct BasicBlockData {
  statements: Vec<Statement>
  terminator: Terminator
}
```

See [Appendix D](#appendix-d--mir-instruction-catalog).

### 10.4 Crate: `rpython_mir_interp` (inside `rpython_mir` as `interp` module)

| File | Responsibility |
|------|----------------|
| `interp/mod.rs` | `interpret(mir_body, args) -> Value` |
| `memory.rs` | Stack slots, heap for `Box` (P9) |
| `ops.rs` | Arithmetic, compare |

Used for tests **before** LLVM is reliable.

---

## 11. Borrow checking and drop elaboration

### 11.1 Crate: `rpython_borrowck`

| File | Responsibility |
|------|----------------|
| `lib.rs` | `borrowck(mir_body) -> Result<BorrowckBody, Diagnostics>` |
| `facts.rs` | Polonius-style or loan-based facts |
| `loans.rs` | `Loan`, `LoanRegion` |
| `moves.rs` | Move path analysis |
| `drops.rs` | Insert `Drop` terminators / statements |
| `region.rs` | Lexical regions in v1 (no full polonius required initially) |

**v1 borrowck model (simplified):**

- Each `Place` has ownership state: **Valid**, **Moved**, **Borrowed { kind, region }**
- Assignment to `mut` place ends loans unless active mutable borrow
- Shared borrow `&T` and mutable `&mut T` follow Rust-like rules
- Return value: `BorrowckBody { body: MirBody, move_errors: [] }`

**Output:** MIR with explicit `Drop` on all control-flow edges where needed.

---

## 12. LLVM codegen

### 12.1 Crate: `rpython_codegen_llvm`

| File | Responsibility |
|------|----------------|
| `lib.rs` | `compile_crate(mir_crate, typed, layout) -> CompiledArtifact` |
| `context.rs` | LLVMContext, Module per crate |
| `mono.rs` | Monomorphization collection |
| `fn.rs` | Function codegen |
| `bb.rs` | Basic block codegen |
| `place.rs` | Place → LLVM value/address |
| `rvalue.rs` | Rvalue lowering |
| `intrinsics.rs` | LLVM intrinsics table |
| `abi.rs` | Calling convention, struct passing |
| `debuginfo.rs` | DIBuilder, line tables (P6+) |
| `link.rs` | Invoke system linker |

### 12.2 `CompiledArtifact`

```text
struct CompiledArtifact {
  object_files: Vec<PathBuf>
  executable: Option<PathBuf>   // if requested
  symbols: Vec<ExportedSymbol>
}
```

### 12.3 Name mangling scheme

```text
_rpy::<crate_hash>::<module_path>::<name>[::<type_args>...]
```

### 12.4 Type → LLVM mapping (defaults)

| rPython type | LLVM type |
|--------------|-----------|
| `bool` | i1 |
| `i8`…`i64`, `u8`…`u64` | integer of width |
| `f32`, `f64` | float |
| `()` | void (empty struct {}) |
| `str` | `{ ptr i8*, len i64 }` |
| `&T` | pointer |
| struct `S` | named struct type |
| enum `E` | `{ tag i32, payload… }` with niche opts later |

---

## 13. Runtime

### 13.1 Crate: `rpython_runtime`

Statically linked Rust runtime crate compiled to `.a` and linked with every binary.

| File | Responsibility |
|------|----------------|
| `lib.rs` | |
| `panic.rs` | `rpy_panic(msg_ptr, len, file, line)` |
| `alloc.rs` | Global allocator hooks if needed |
| `str.rs` | UTF-8 helpers, panic on invalid |
| `io.rs` | `stdout_write`, `stderr_write` stubs |
| `init.rs` | `rpy_rt_init`, `rpy_rt_fini` |

### 13.2 Runtime symbols (exported C ABI)

| Symbol | Purpose |
|--------|---------|
| `rpy_panic` | Abort with message |
| `rpy_write_stdout` | Buffered write |
| `rpy_str_eq` | String equality |
| `rpy_int_to_string` | Debug printing (tests) |

User `main` lowers to `fn main() -> i32` calling `rpy_user_main`.

---

## 14. Driver and CLI

### 14.1 Crate: `rpython_driver`

| File | Responsibility |
|------|----------------|
| `lib.rs` | `CompilerSession`, `compile` |
| `session.rs` | Options, file graph |
| `pipeline.rs` | Ordered passes |
| `cache.rs` | Incremental cache (P10) — hash source + deps |

**Pipeline order:**

```text
1. load_sources
2. parse_all → AstCrate
3. resolve → Resolution
4. typecheck → TypedProgram
5. build_hir → HirCrate
6. monomorphize → MonoPlan
7. build_mir → MirCrate
8. borrowck (per function)
9. codegen_llvm
10. link
```

### 14.2 Crate: `rpython_cli`

Binary name: **`rpythonc`**

| Flag | Effect |
|------|--------|
| `--emit tokens` | Stop after lex; print tokens |
| `--emit ast` | Parse only |
| `--emit hir` | Through HIR |
| `--emit mir` | Through MIR |
| `--emit llvm` | LLVM IR to stdout |
| `-o PATH` | Output executable |
| `--crate-type bin\|lib` | Artifact kind |
| `-g` | Debug info |
| `--test` | Build test harness main |
| `-O0`…`-O3` | LLVM opt level |
| `--explain E0123` | Print error doc |

---

## 15. Standard library

Sources live in `stdlib/`. Compiled as **rlib** crate type with name from `rpython.toml`.

### 15.1 `stdlib/prelude.rpy`

Re-exports: `Option`, `Result`, `Vec`, `print`, `panic`, primitive types.

### 15.2 `stdlib/core/`

| Module file | Public items |
|-------------|--------------|
| `option.rpy` | `enum Option[T]: Some(T), None` |
| `result.rpy` | `enum Result[T, E]: Ok(T), Err(E)` |
| `ops.rpy` | Trait definitions for operators |
| `mem.rpy` | `size_of`, `align_of` |
| `convert.rpy` | `From`, `Into` traits |

### 15.3 `stdlib/collections/`

| Module | Items |
|--------|-------|
| `vec.rpy` | `struct Vec[T]`, `push`, `pop`, `len` |
| `map.rpy` | `struct Map[K, V]` (open addressing) |
| `set.rpy` | `struct Set[T]` |

### 15.4 `stdlib/io/`

| Module | Items |
|--------|-------|
| `file.rpy` | `File`, `read`, `write` |
| `path.rpy` | `Path`, `join` |
| `stdout.rpy` | `print`, `println` |

**Implementation note:** stdlib functions are mostly `extern` to runtime Rust for I/O until self-hosting.

---

## 16. Diagnostics and error codes

### 16.1 Code ranges

| Range | Domain |
|-------|--------|
| E0001–E0099 | Lexer/parser |
| E0100–E0199 | Resolution |
| E0200–E0299 | Name/bindings |
| E0300–E0399 | Types/traits |
| E0400–E0499 | Borrowck |
| E0500–E0599 | MIR/lowering |
| E0600–E0699 | Codegen/link |

### 16.2 Example entries (implement `docs/errors/E0306.md`)

- **E0306:** trait bound not satisfied — include obligation, candidate impls tried.

### 16.3 JSON diagnostic shape

```text
{
  "message": "...",
  "code": "E0306",
  "level": "error",
  "spans": [{ "file": "...", "line_start": 1, "col_start": 4, ... }],
  "children": [...]
}
```

---

## 17. Package, module, and build model

### 17.1 `rpython.toml`

```text
[package]
name = "myapp"
version = "0.1.0"
edition = "2024"

[dependencies]
collections = { path = "../stdlib/collections" }

[[bin]]
name = "myapp"
path = "src/main.rpy"
```

### 17.2 Module path rules

- File `src/foo/bar.rpy` → module `foo.bar`
- `import foo.bar` resolves via `RPYTHON_PATH` and dependency crates
- `pub` on items controls visibility

### 17.3 Crate types

| Type | Output |
|------|--------|
| `bin` | Executable |
| `lib` | `.rlib` archive of MIR/metadata + object |

---

## 18. Test harness and fixtures

### 18.1 Crate: `rpython_test_runner`

```text
fn run_tests(path: &Path, session: &CompilerSession) -> TestReport
```

- Discovers `#[test]` functions (attribute on `def`)
- Compiles each with `--test` shim calling all tests
- Captures stdout/stderr vs `.stdout.expect`

### 18.2 Fixture conventions

| Directory | Input | Expected |
|-----------|-------|----------|
| `tests/lexer/` | `foo.rpy` | `foo.tokens.expect` |
| `tests/parser/` | `foo.rpy` | `foo.ast.json.expect` |
| `tests/typeck/` | `foo.rpy` | `foo.stderr.expect` |
| `tests/mir/` | `foo.rpy` | `foo.mir.expect` |
| `tests/ui/run-pass/` | `foo.rpy` | exit 0 |
| `tests/ui/compile-fail/` | `foo.rpy` | exact stderr |

---

## 19. Phased delivery map

Each phase lists **crates/files to create or complete** and **acceptance criteria**. Phases are cumulative.

### P0 — Workspace bootstrap

**Deliver:** root `Cargo.toml`, empty crates with `lib.rs`, CI workflow, `examples/hello.rpy` (comment only).

**Acceptance:** `cargo build` and `cargo test` succeed (zero tests ok).

---

### P1 — Lexer (M0)

**Files:** `rpython_syntax/*`, span/errors/ids crates.

**Acceptance:** `rpythonc --emit tokens` on fixtures; INDENT/DEDENT correct; error spans on invalid chars.

---

### P2 — Parser + AST (M1)

**Files:** `rpython_ast/*`, `rpython_parse/*`, `grammar.md`.

**Acceptance:** Parse micro-grammar; AST JSON snapshots; pretty-print round-trip on `tests/parser/*`.

**Micro-grammar scope:** literals, `def`, `return`, `if`, `while`, assignments, binary ops, calls.

---

### P3 — Name resolution (M2)

**Files:** `rpython_resolve/*`.

**Acceptance:** Resolve imports in multi-file crate; duplicate def errors; symbol map on paths.

---

### P4 — Typechecker core (M3)

**Files:** `rpython_types/*`, `rpython_typeck/*`, start `docs/LANGUAGE.md`.

**Acceptance:** Typecheck `int`, `bool`, functions, `if`, `while`, `return`; 50+ `tests/typeck` fixtures.

---

### P5 — HIR + MIR + interpreter (M4)

**Files:** `rpython_hir*`, `rpython_mir*`, `interp/*`.

**Acceptance:** Execute `tests/mir/*` via interpreter; HIR/MIR pretty snapshots.

---

### P6 — LLVM + runtime + driver (M5)

**Files:** `rpython_codegen_llvm/*`, `rpython_runtime/*`, `rpython_driver/*`, `rpython_cli/*`.

**Acceptance:** `rpythonc examples/hello.rpy -o hello && ./hello` prints expected; no panic on invalid input.

---

### P7 — Structs, enums, impls (M6)

**Extend:** AST, resolve, typeck, layout, MIR aggregates, codegen ABI.

**Acceptance:** Struct construction, enum `match`, `impl` methods, trait objects **not** required yet.

---

### P8 — Traits + monomorphization

**Extend:** `trait.rs`, impl coherence, vtable-free dispatch.

**Acceptance:** `traits_demo.rpy` compiles; static dispatch on generics.

---

### P9 — Borrowck + drops

**Files:** `rpython_borrowck/*`; MIR `Drop` terminators.

**Acceptance:** Move errors; `&mut` exclusivity; slice/string borrow rules.

---

### P10 — Stdlib v0 + test runner (M7)

**Files:** `stdlib/*`, `rpython_test_runner/*`.

**Acceptance:** `rpython test` runs stdlib unit tests; `Vec`, `print`, `File` basics.

---

### P11 — Language surface completion

**Add:** `match` patterns full, `for`, `loop`, `break`/`continue`, `enum`/`struct`/`class`, `pub`, modules, `extern`, attributes, integer widths, `bytes`, string formatting subset.

**Acceptance:** Language reference `docs/LANGUAGE.md` matches compiler; UI tests cover each feature.

---

### P12 — Tooling hardening

**Add:** incremental cache, `-g` DWARF, `cargo install`, package publishing story, bench suite.

**Acceptance:** CI on Linux/macOS; release workflow produces `rpythonc` binary.

---

## Appendix A — Token catalog

```text
enum TokenKind {
  // End / structure
  Eof
  Newline
  Indent
  Dedent

  // Literals
  IntLit { value: IntLiteral }      // decimal; 0x hex; 0b bin
  FloatLit { value: f64 }
  StringLit { value: String }       // unicode escapes \n \t \u{...}
  BytesLit { value: Vec<u8> }
  BoolLit(bool)

  // Identifiers / keywords
  Ident { name: SmolStr }
  KwDef, KwClass, KwEnum, KwStruct, KwImpl, KwTrait
  KwIf, KwElif, KwElse, KwWhile, KwFor, KwLoop, KwBreak, KwContinue
  KwReturn, KwPass, KwImport, KwFrom, KwAs, KwPub
  KwMut, KwRef, KwTrue, KwFalse, KwNone, KwSelf, KwSuper
  KwIn, KwIs, KwNot, KwAnd, KwOr
  KwExtern, KwUnsafe, KwAsync(reserved), KwAwait(reserved)

  // Operators (representative)
  Plus, Minus, Star, Slash, Percent, FloorDiv, Pow
  EqEq, NotEq, Lt, LtEq, Gt, GtEq
  Assign, PlusAssign, MinusAssign, ...
  Arrow, FatArrow, Colon, Semi, Comma, Dot, DotDot, DotDotEq
  LParen, RParen, LBracket, RBracket, LBrace, RBrace
  At, Hash, Underscore
  Amp, AmpMut, Pipe, Bang, Tilde, Question
  Ellipsis

  // Comments/whitespace handled in lexer, not emitted
}
```

---

## Appendix B — AST node catalog

### B.1 `ItemKind`

```text
enum ItemKind {
  Function { name, generics, params, ret_ty, body, is_pub, attrs }
  Class { name, generics, bases, body, is_pub, attrs }
  Struct { name, generics, fields, is_pub, attrs }
  Enum { name, generics, variants, is_pub, attrs }
  Trait { name, generics, items, is_pub, attrs }
  Impl { generics, trait_ref, self_ty, items, attrs }
  Const { name, ty, value, is_pub }
  Import { path, alias }
  ExternBlock { abi, items }
  Module { name, items }   // inline modules
}
```

### B.2 `StmtKind`

```text
enum StmtKind {
  Expr(ExprId)
  Assign { targets: Vec<PatId>, value: ExprId }
  AnnAssign { target: PatId, ty: TyId, value: Option<ExprId> }
  Return(Option<ExprId>)
  Raise(ExprId)              // desugar to panic early; later Result
  Assert { test, msg }
  Pass
  Break(Option<Label>)
  Continue(Option<Label>)
  While { test, body }
  For { pat, iter, body }
  If { test, then_body, elifs, else_body }
  Match { scrutinee, arms }
}
```

### B.3 `ExprKind`

```text
enum ExprKind {
  Literal(Literal)
  Path(Path)
  Call { func, args, kwargs }   // kwargs restricted to typed dict builder
  MethodCall { receiver, method, args }
  Field { base, field }
  Index { base, index }
  Unary { op, operand }
  Binary { op, left, right }
  Tuple(Vec<ExprId>)
  List(Vec<ExprId>)
  Struct { path, fields }
  If { test, then, else }
  Block(Vec<StmtId>)            // inline blocks in expressions (rare)
  Lambda { params, body }       // P11
  Cast { expr, ty }
  Ref { mutability, expr }
  Deref(ExprId)
}
```

### B.4 `PatKind`

```text
enum PatKind {
  Wild
  Ident { name, mut, subpat: Option<PatId> }
  Literal(Literal)
  Tuple(Vec<PatId>)
  Struct { path, fields }
  Enum { path, variant, subpats }
  Or(Vec<PatId>)
}
```

### B.5 `TyKind` (syntax)

```text
enum TyKind {
  Path(Path)
  Tuple(Vec<TyId>)
  Array { elem, len }
  Slice { elem }
  Ref { mutability, inner }
  Fn { params, ret }
  GenericParam { name }
}
```

---

## Appendix C — HIR node catalog

```text
enum HirExprKind {
  Literal(HirLiteral)
  Path { def: DefId, subst: Subst }
  Unary { op, operand: HirExprId }
  Binary { op, left, right }
  Call { def: DefId, subst, args: Vec<HirExprId> }
  MethodCall { trait_method: DefId, subst, receiver, args }
  Field { base, field_index }
  Index { base, index }
  AddrOf { mutability, place: Place }
  Deref(Place)
  Cast { expr, ty }
  If { cond, then, else }
  Match { scrutinee, arms: Vec<(HirPat, HirExprId)> }
  Tuple(Vec<HirExprId>)
  Struct { def, fields }
}

enum HirStmtKind {
  Assign { place, rvalue }
  Expr(HirExprId)
  Return(Option<HirExprId>)
  Drop(Place)                    // explicit post-borrowck
}
```

**Place / Rvalue (HIR):**

```text
enum PlaceKind {
  Local(LocalId)
  Param(u32)
  Field { base: Place, idx }
  Index { base: Place, index: HirExprId }
  Deref(Place)
}

enum Rvalue {
  Use(Operand)
  UnaryOp { op, operand }
  BinaryOp { op, left, right }
  Aggregate(AggregateKind)
  Ref { mutability, place }
  Len(Place)
  Discriminant(Place)
}
```

---

## Appendix D — MIR instruction catalog

### D.1 `StatementKind`

```text
enum StatementKind {
  Assign { place: Place, rvalue: Rvalue }
  StorageLive(LocalId)
  StorageDead(LocalId)
  Deinit(Place)              // after move
  Nop
}
```

### D.2 `Rvalue` (MIR)

```text
enum Rvalue {
  Use(Operand)
  UnaryOp { op, operand }
  BinaryOp { op, left, right }
  CheckedBinaryOp { ... }     // optional for overflow hooks
  Aggregate { kind, ops: Vec<Operand> }
  Ref { region, mutability, place }
  Len(Place)
  Cast { kind: CastKind, operand, ty }
  Discriminant(Place)
  Repeat { operand, len }
}
```

### D.3 `TerminatorKind`

```text
enum TerminatorKind {
  Goto { target: BlockId }
  SwitchInt { discr, targets: Vec<(u128, BlockId)>, otherwise }
  Return
  Unreachable
  Drop { place, target, unwind }   // unwind = None in v1 (abort)
  Call { func, args, destination, target, unwind }
}
```

### D.4 `Projection` (MIR place projections)

```text
enum Projection {
  Field(u32)
  Index(LocalId)           // index local
  Deref
  Downcast(u32)            // enum variant
}
```

### D.5 `Operand`

```text
enum Operand {
  Copy(Place)
  Move(Place)
  Constant(ConstValue)
}
```

---

## Appendix E — Type kind catalog

```text
enum TyKind {
  Bool
  Int(IntWidth)            // I8…I64, U8…U64
  Float(FloatWidth)
  Char                     // Unicode scalar
  Str                      // owned UTF-8
  Bytes                    // owned bytes
  Unit
  Never                    // !
  Tuple(Vec<TypeId>)
  Array { elem: TypeId, len: usize }
  Slice { elem: TypeId }
  Ref { mutability, elem: TypeId, region: RegionId }
  Adt { def: DefId, subst: Subst }   // struct/enum/class
  FnDef { def: DefId, subst: Subst }
  FnPtr { sig: FnSig }
  TraitObject { trait_ref: TraitRef }  // P8+ vtables
  Infer(InferVar)
  Error                    // poison after diagnostic
  GenericParam { index: u32 }
}
```

**Subst:**

```text
struct Subst {
  args: Vec<TypeId>
}
```

---

## Codegen usage notes

When using this spec with an LLM or codegen tool:

1. **Generate crates bottom-up:** `rpython_ids` → `rpython_span` → `rpython_errors` → … → `rpython_cli`.
2. **Generate tests alongside each crate** using fixture tables in §18.
3. **Do not merge passes** — each crate boundary is a compile-time dependency gate.
4. **Snapshots:** use `insta` or `serde_json` for AST/MIR; normalize paths in tests.
5. **Feature flags:** `cranelift` backend, `async`, `gc` remain **off** until their phase.

---

**End of implementation specification.** Update this document when RFCs change semantics; keep `DESIGN_SPEC.md` as product intent and this file as build truth.
