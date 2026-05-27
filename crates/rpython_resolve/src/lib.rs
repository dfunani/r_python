//! Name resolution: collect definitions, resolve paths, handle imports.

mod collect;
mod def_map;
mod imports;
mod resolve_expr;
mod resolved;
mod ribs;
mod scope;
mod symbols;

pub use def_map::{DefKind, DefMap};
pub use imports::ImportRecord;
pub use resolved::{resolve_path, ResolvedCrate};
pub use scope::{Scope, ScopeKind};
pub use symbols::{Binding, NameBinding};

use collect::Collector;
use def_map::DefKind as DK;
use imports::ImportRecord as IR;
use resolve_expr::ExprResolver;
use ribs::RibStack;
use rpython_ast::{Arena, ItemKind, Module, Path};
use rpython_errors::{Diagnostic, ErrorCode, Handler};
use rpython_ids::{CrateId, DefId, ExprId, ModuleId, SymbolId};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

/// Tree of modules in a crate (single-file v1).
#[derive(Clone, Debug, Default)]
pub struct ModuleTree {
    pub root: ModuleId,
    pub names: FxHashMap<ModuleId, SmolStr>,
}

/// Well-known builtin definition ids.
#[derive(Clone, Copy, Debug)]
pub struct BuiltinDefs {
    pub ty_int: DefId,
    pub ty_bool: DefId,
    pub ty_str: DefId,
    pub ty_unit: DefId,
    pub ty_float: DefId,
    pub print: DefId,
}

/// Output of name resolution.
#[derive(Clone, Debug)]
pub struct Resolution {
    pub def_map: DefMap,
    pub symbol_map: FxHashMap<ExprId, SymbolId>,
    pub expr_bindings: FxHashMap<ExprId, NameBinding>,
    pub module_tree: ModuleTree,
    pub imports: Vec<ImportRecord>,
    pub crate_id: CrateId,
    pub builtins: BuiltinDefs,
}

/// Resolve names in `module` using `arena` for AST nodes.
pub fn resolve_crate(module: &Module, arena: &Arena, handler: &mut Handler) -> Option<Resolution> {
    let crate_id = CrateId(0);
    let root_module = ModuleId(0);
    let mut def_map = DefMap::new(root_module);
    let mut ribs = RibStack::new();
    let parent = def_map.root_def();

    let builtins = inject_builtins(&mut def_map, parent, &mut ribs);

    {
        let mut collector = Collector {
            def_map: &mut def_map,
            ribs: &mut ribs,
            handler,
            parent,
        };
        collector.collect_module(module, arena);
    }

    let mut imports = Vec::new();
    process_imports(
        module,
        arena,
        &mut def_map,
        &mut ribs,
        handler,
        &mut imports,
    );

    let mut expr_bindings = FxHashMap::default();
    let mut resolver = ExprResolver {
        def_map: &mut def_map,
        ribs: &mut ribs,
        handler,
        symbol_map: &mut expr_bindings,
        current_fn: None,
        module_parent: parent,
    };
    resolver.resolve_module(&module.items, arena);

    let mut symbol_map = FxHashMap::default();
    for (expr, binding) in &expr_bindings {
        symbol_map.insert(*expr, SymbolId(binding.def.0));
    }

    if handler.has_errors() {
        return None;
    }

    let resolution = Resolution {
        def_map,
        symbol_map,
        expr_bindings,
        module_tree: ModuleTree {
            root: root_module,
            names: FxHashMap::default(),
        },
        imports,
        crate_id,
        builtins,
    };

    Some(resolution)
}

/// Resolve and wrap as `ResolvedCrate` for type checking.
pub fn resolve_for_typeck(
    module: &Module,
    arena: &Arena,
    handler: &mut Handler,
) -> Option<ResolvedCrate> {
    let resolution = resolve_crate(module, arena, handler)?;
    Some(ResolvedCrate::from_resolution(resolution, module, arena))
}

fn inject_builtins(def_map: &mut DefMap, parent: DefId, ribs: &mut RibStack) -> BuiltinDefs {
    ribs.push(scope::ScopeKind::Root, parent, None);

    let ty_int = def_map.alloc(DK::BuiltinType { name: "int".into() });
    def_map.insert_name(parent, "int".into(), ty_int);
    ribs.define("int".into(), ty_int);

    let ty_bool = def_map.alloc(DK::BuiltinType {
        name: "bool".into(),
    });
    def_map.insert_name(parent, "bool".into(), ty_bool);
    ribs.define("bool".into(), ty_bool);

    let ty_str = def_map.alloc(DK::BuiltinType { name: "str".into() });
    def_map.insert_name(parent, "str".into(), ty_str);
    ribs.define("str".into(), ty_str);

    let ty_unit = def_map.alloc(DK::BuiltinType {
        name: "void".into(),
    });
    def_map.insert_name(parent, "void".into(), ty_unit);
    ribs.define("void".into(), ty_unit);

    let ty_float = def_map.alloc(DK::BuiltinType {
        name: "float".into(),
    });
    def_map.insert_name(parent, "float".into(), ty_float);
    ribs.define("float".into(), ty_float);

    let print = def_map.alloc(DK::BuiltinFn {
        name: "print".into(),
    });
    def_map.insert_name(parent, "print".into(), print);
    ribs.define("print".into(), print);

    ribs.pop();

    BuiltinDefs {
        ty_int,
        ty_bool,
        ty_str,
        ty_unit,
        ty_float,
        print,
    }
}

fn process_imports(
    module: &Module,
    arena: &Arena,
    def_map: &mut DefMap,
    ribs: &mut RibStack,
    handler: &mut Handler,
    imports: &mut Vec<ImportRecord>,
) {
    ribs.push(scope::ScopeKind::Root, def_map.root_def(), None);
    for &item_id in &module.items {
        let item = arena.item(item_id);
        if let ItemKind::Import { path, alias } = &item.kind {
            let resolved = resolve_import_path(path, def_map, def_map.root_def());
            let record = IR::from_path(path, alias.clone(), item.span, resolved.is_some());
            if let Some(target) = resolved {
                let alias_name = alias
                    .clone()
                    .or_else(|| path.segments.last().map(|s| s.ident.clone()))
                    .unwrap_or_else(|| "unknown".into());
                def_map.insert_name(def_map.root_def(), alias_name.clone(), target);
                ribs.define(alias_name, target);
            } else {
                handler.emit(
                    Diagnostic::error(format!(
                        "unresolved import `{}`",
                        path.segments
                            .iter()
                            .map(|s| s.ident.as_str())
                            .collect::<Vec<_>>()
                            .join(".")
                    ))
                    .with_code(ErrorCode::E0203)
                    .with_label(item.span, "could not resolve import", true),
                );
            }
            imports.push(record);
        }
    }
    ribs.pop();
}

fn resolve_import_path(path: &Path, def_map: &DefMap, root: DefId) -> Option<DefId> {
    if path.segments.is_empty() {
        return None;
    }
    let mut current = def_map.lookup(root, &path.segments[0].ident)?;
    for seg in path.segments.iter().skip(1) {
        current = def_map.lookup(current, &seg.ident)?;
    }
    Some(current)
}

/// Resolve `from module import name`.
pub fn resolve_from_import(
    module_path: &Path,
    name: &str,
    def_map: &DefMap,
    root: DefId,
) -> Option<DefId> {
    let module = resolve_import_path(module_path, def_map, root)?;
    def_map.lookup(module, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rpython_ast::{ExprKind, ItemKind, Literal, StmtKind};
    use rpython_span::{BytePos, FileId, Span};

    fn span() -> Span {
        Span::new(FileId(0), BytePos(0), BytePos(1))
    }

    #[test]
    fn resolve_simple_function() {
        let mut arena = Arena::new();
        let lit = arena.alloc_expr(ExprKind::Literal(Literal::Int(1)), span());
        let ret = arena.alloc_stmt(StmtKind::Return(Some(lit)), span());
        let func = arena.alloc_item(
            ItemKind::Function {
                name: "main".into(),
                generics: vec![],
                params: vec![],
                ret_ty: None,
                body: vec![ret],
                is_pub: true,
                attrs: vec![],
            },
            span(),
        );
        let module = Module {
            items: vec![func],
            span: span(),
        };
        let mut handler = Handler::new();
        let res = resolve_crate(&module, &arena, &mut handler).unwrap();
        assert!(!handler.has_errors());
        assert!(res.def_map.lookup(res.def_map.root_def(), "main").is_some());
        assert!(res
            .def_map
            .lookup(res.def_map.root_def(), "print")
            .is_some());
    }
}
