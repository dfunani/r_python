# rPython language reference (v2.0 implemented subset)

**Status:** Living document for the **current compiler** (v2.0.0).  
**Full target:** [DESIGN_SPEC.md](./DESIGN_SPEC.md) · [IMPLEMENTATION.md](./IMPLEMENTATION.md)  
**Build status:** [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md)

---

## What works today (v2.0)

Single-file `.rpy` programs compile and run via **`rpythonc run`** (MIR interpreter) or **`rpythonc build -o`** (C backend).

### Statements and control flow

- `def`, `return`, assignment (including reassignment of locals/parameters)
- `if` / `elif` / `else`, `while`
- Expression statements

### Types (static — not dynamic)

rPython is **statically typed**. Types are checked at compile time. Example:

```rpy
a: str = "hello"   # ok
a: int = "hello"   # compile error: expected `int`, found `str`
```

This is **not** CPython-style dynamic typing.

| Type | Notes |
|------|--------|
| `int` | i64 |
| `bool` | |
| `str` | UTF-8 literals |
| `()` | unit |

**Annotated locals** inside functions: `name: Type = value` (v2.0).

### Expressions

- Literals, paths, calls, unary/binary (`+ - * / %`, comparisons, `and` / `or` / `not`)
- **Struct literals:** `Point { x: 1, y: 2 }`
- **Method calls:** `p.show()` (static dispatch via `impl` or class methods)
- **Constructor calls:** `Greeter()` for classes/structs

### Items (top level)

| Item | Status |
|------|--------|
| `def` | **works** |
| `struct` | **works** (field literals, POD) |
| `class` | **partial** (methods; field annotations planned) |
| `interface` | **works** (signatures; no `...` body — use signature-only methods) |
| `trait` | deprecated alias for `interface` |
| `impl Interface for Type` | **works** (static method dispatch) |
| `enum`, `match`, `for` | parse only / partial typeck — not e2e |
| `import` / modules | single-file only |

### Builtins

| Name | Behavior |
|------|----------|
| `print` | one value: `int`, `bool`, `str`, or debug representation |

### Stdlib (source scaffold)

Sources under `stdlib/` (`core/option.rpy`, `collections/vec.rpy`) are **not** yet loaded automatically; use builtins for v2.0 programs.

### v2 examples (all run with `rpythonc run`)

- `examples/hello.rpy`
- `examples/static_typing.rpy`
- `examples/gcd.rpy`
- `examples/interfaces_demo.rpy`
- `examples/classes_demo.rpy`
- `examples/traits_demo.rpy`

---

## Interfaces and classes (v2)

```rpy
interface Show:
    def show(self) -> str

struct Point:
    x: int
    y: int

impl Show for Point:
    def show(self) -> str:
        return "Point"

class Greeter:
    def greet(self) -> int:
        print("hello from Greeter")
        return 0
```

- Prefer **`interface`** over deprecated **`trait`**.
- Use **`class`** for behavior + state; **`struct`** for plain data.
- Do not use `...` in interface bodies (parser may hang); use signature-only methods.

---

## CLI

```bash
rpythonc run program.rpy
rpythonc build -o ./app program.rpy
rpythonc --emit mid-level-ir program.rpy   # alias: mir
rpythonc --emit high-level-ir program.rpy  # alias: hir
```

---

## Not yet reliable

- Multi-file crates and `import` graph
- `match`, `for`, `break`/`continue` through codegen
- Full borrow-checker diagnostics (move/`&mut` errors)
- LLVM backend (C is the production bootstrap backend)
- Automatic `stdlib/` prelude loading

See [V2_ROADMAP.md](./V2_ROADMAP.md) for remaining P3–P12 work.
