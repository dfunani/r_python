//! Discover and run `#[test]` functions via the MIR interpreter.

use std::path::Path;

use anyhow::{Context, Result};
use rpython_ast::{Arena, Attribute, ItemKind};
use rpython_errors::Handler;
use rpython_mir::interp::interpret;
use rpython_mir_build::build_mir_from_typed;
use rpython_parse::parse_module;
use rpython_resolve::resolve_crate;
use rpython_span::SourceMap;
use rpython_syntax::tokenize;
use rpython_typeck::typecheck;
use smol_str::SmolStr;

/// Report from running tests in a source file.
#[derive(Clone, Debug, Default)]
pub struct TestReport {
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<TestFailure>,
}

#[derive(Clone, Debug)]
pub struct TestFailure {
    pub name: SmolStr,
    pub message: String,
}

/// Discover `#[test]` functions in `path` and run each via MIR interpretation.
pub fn run_tests(path: &Path) -> Result<TestReport> {
    let contents =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut source_map = SourceMap::new();
    let file_id = source_map.load_file(path, contents);
    let mut handler = Handler::new();
    let stream = tokenize(&source_map, file_id, &mut handler);
    if handler.has_errors() {
        anyhow::bail!("lex errors in {}", path.display());
    }

    let arena = Arena::new();
    let module = parse_module(stream, &arena, &mut handler)
        .ok_or_else(|| anyhow::anyhow!("parse failed"))?;
    if handler.has_errors() {
        anyhow::bail!("parse errors in {}", path.display());
    }

    let resolution = resolve_crate(&module, &arena, &mut handler)
        .ok_or_else(|| anyhow::anyhow!("resolve failed"))?;
    let typed = typecheck(&resolution, &module, &arena, &mut handler)
        .ok_or_else(|| anyhow::anyhow!("typecheck failed"))?;
    if handler.has_errors() {
        anyhow::bail!("type errors in {}", path.display());
    }

    let mir = build_mir_from_typed(&typed, &module, &arena);

    let mut report = TestReport::default();
    for (&func_id, func) in &mir.functions {
        let item = module
            .items
            .iter()
            .find_map(|&id| {
                let item = arena.item(id);
                if let ItemKind::Function { name, attrs, .. } = &item.kind {
                    if name == &func.name && has_test_attr(attrs) {
                        return Some(name.clone());
                    }
                }
                None
            });

        if item.is_none() && !has_test_attr_on_fn(&arena, &module, &func.name) {
            continue;
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            interpret(&mir, func_id, vec![]);
        }));
        match result {
            Ok(_) => report.passed += 1,
            Err(payload) => {
                let message = payload
                    .downcast_ref::<String>()
                    .map(|s| s.clone())
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| "panic".into());
                report.failed += 1;
                report.failures.push(TestFailure {
                    name: func.name.clone(),
                    message,
                });
            }
        }
    }

    Ok(report)
}

fn has_test_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| a.name.as_str() == "test")
}

fn has_test_attr_on_fn(arena: &Arena, module: &rpython_ast::Module, name: &str) -> bool {
    for &id in &module.items {
        let item = arena.item(id);
        if let ItemKind::Function {
            name: n,
            attrs,
            ..
        } = &item.kind
        {
            if n.as_str() == name && has_test_attr(attrs) {
                return true;
            }
        }
    }
    false
}
