# Diagnostic error codes

Stable codes are defined in `crates/rpython_errors/src/codes.rs`.  
CLI: `rpythonc --explain E0300`

| Code | Summary | Detail |
|------|---------|--------|
| [E0001](./E0001.md) | Invalid character | Lexer |
| [E0002](./E0002.md) | Unterminated string | Lexer |
| [E0201](./E0201.md) | Duplicate definition | Resolve |
| [E0202](./E0202.md) | Used before definition | Resolve |
| [E0203](./E0203.md) | Unresolved import | Resolve |
| [E0204](./E0204.md) | Cannot resolve name | Resolve |
| [E0300](./E0300.md) | Type mismatch | Typeck |
| [E0301](./E0301.md) | Wrong argument count | Typeck |
| [E0302](./E0302.md) | Cannot infer type | Typeck |
| [E0303](./E0303.md) | Non-exhaustive match | Typeck |
| [E0304](./E0304.md) | Wrong return type | Typeck |
| [E0305](./E0305.md) | Ambiguous resolution | Typeck |
| [E0306](./E0306.md) | Trait bound not satisfied | Typeck |

**Future:** `tools/gen_errors.rs` should generate these pages from the registry (see [tools/README.md](../../tools/README.md)).
