# rPython language reference (implemented subset)

**Status:** Living document for the **current compiler**, not the full v1 target.  
**Full target:** [DESIGN_SPEC.md](./DESIGN_SPEC.md) · [IMPLEMENTATION.md](./IMPLEMENTATION.md)  
**Build status:** [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md)

---

## What works today (2026-05-27)

The compiler accepts **single-file** `.rpy` programs with:

- Indentation-based blocks (`INDENT` / `DEDENT` from the lexer)
- Top-level `def` functions with optional `-> type` return annotation
- Statements: expression stmts, `return`, `if` / `elif` / `else`, `while`, assignment to simple names
- Expressions: integer and string literals, `bool` literals, binary operators (`+ - * / %`, comparisons, `and` / `or` / `not`), calls, paths
- Builtin **`print`** (one argument: `int`, `bool`, or `str`)

Types known to the typechecker (partial):

| Type | Notes |
|------|--------|
| `int` | i64 default |
| `bool` | two-valued |
| `str` | UTF-8 string literals |
| `()` | unit where inferred |

**Not reliable end-to-end yet** (may parse or typecheck in places but not codegen/interpreter): `struct`, `enum`, `match`, `impl`, `trait`, `class`, generics, `for`, `break`/`continue`, modules/imports across files, `bytes`, references `&` / `&mut`, method calls, aggregates.

---

## Lexical structure

- **Comments:** `#` to end of line (not tokens)
- **Strings:** `"..."` with escapes `\n`, `\t`, `\\`, `\"`
- **Integers:** decimal (other bases per lexer may parse; typeck may be narrow)
- **Identifiers:** Unicode letters/digits; keywords reserved

See [Appendix A in IMPLEMENTATION.md](./IMPLEMENTATION.md#appendix-a--token-catalog) for the full target token set.

---

## Grammar (implemented micro-subset)

```text
file        ::= item*
item        ::= 'def' name '(' params? ')' ('->' type)? ':' block
block       ::= stmt+
stmt        ::= simple_stmt | if_stmt | while_stmt
simple_stmt ::= expr | 'return' expr? | name '=' expr
if_stmt     ::= 'if' expr ':' block ('elif' expr ':' block)* ('else' ':' block)?
while_stmt  ::= 'while' expr ':' block
expr        ::= literal | name | call | unary | binary
call        ::= expr '(' args? ')'
```

Parser accepts **more** than this (structs, enums, etc.); only the subset above is validated for compilation to MIR/C.

---

## Types and functions

```rpy
def add(x: int, y: int) -> int:
    return x + y

def main() -> int:
    print(add(1, 2))
    return 0
```

- Parameters and return types use the **annotation** grammar (`type` paths).
- Missing return type on `main` may default or error depending on context; prefer explicit `-> int` for entry.

---

## Builtins

| Name | Signature (conceptual) | Behavior |
|------|------------------------|----------|
| `print` | `print(value)` | Writes `int` / `bool` / `str` to stdout; adds newline |

---

## Compilation

```bash
# Tokens / AST / MIR (debug)
cargo run -p rpython_cli -- --emit tokens examples/hello.rpy
cargo run -p rpython_cli -- --emit ast examples/hello.rpy
cargo run -p rpython_cli -- --emit mir examples/hello.rpy

# Interpret (no native code)
cargo run -p rpython_cli -- --run examples/hello.rpy

# Native executable (C backend + platform linker)
cargo run -p rpython_cli -- -o ./hello examples/hello.rpy
./hello
```

---

## Planned (not in this document’s guarantees)

Listed in phase order in [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md): ownership/borrowing, structs/enums, traits, stdlib, modules, `match`/`for`, LLVM backend, `rpython test`, package manifests.

When a feature moves from “planned” to “implemented”, add a subsection here and a test under `tests/`.
