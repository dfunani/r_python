# rPython standard library (v2 scaffold)

Source libraries for the rPython language. The compiler **v2.0** ships these as reference sources; automatic prelude loading is planned for P10.

## Layout

| Path | Purpose |
|------|---------|
| `core/prelude.rpy` | Implicit imports (documented) |
| `core/option.rpy` | `Option[T]` enum |
| `collections/vec.rpy` | `Vec[T]` growable buffer (runtime-backed later) |

## Builtins (compiler)

Provided by `rpython_resolve` today: `print`, type names `int`, `bool`, `str`, `void`.

## Running tests

When `rpythonc test` is wired to load `stdlib/`, programs will `import` from here. Until then, copy snippets into your `.rpy` file or use builtins only.
