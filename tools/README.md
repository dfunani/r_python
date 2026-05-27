# Developer tools

**Status:** Planned per [IMPLEMENTATION.md](../docs/IMPLEMENTATION.md) §2.

| Tool | Purpose | Status |
|------|---------|--------|
| `gen_errors.rs` | Generate `docs/errors/*.md` from `rpython_errors::codes` | not started |
| `gen_ast_visitor.rs` | Optional AST visitor codegen | not started |

These binaries are **not** workspace members yet. Add under `tools/` and `[[bin]]` in root `Cargo.toml` when implementing P12 doc automation.
