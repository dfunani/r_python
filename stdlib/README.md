# rPython standard library (P10)

**Status:** Not implemented. This directory is reserved per [IMPLEMENTATION.md](../docs/IMPLEMENTATION.md) §15.

## Planned layout

```text
stdlib/
├── core/           # Option, Result, panic hooks
├── collections/    # Vec, HashMap (later)
├── io/             # File, stdin/stdout abstractions
└── prelude.rpy     # names imported implicitly (design TBD)
```

## Current builtins

The compiler embeds a minimal builtin set in `rpython_resolve` (e.g. `print`). Until stdlib sources compile, builtins live in Rust.

## Tracking

See [docs/IMPLEMENTATION_STATUS.md](../docs/IMPLEMENTATION_STATUS.md) — phase **P10**.
