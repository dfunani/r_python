use rpython_ast::{Arena, ItemId, ItemKind, Module, Path};
use rpython_ids::{DefId, ExprId};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

use crate::symbols::NameBinding;
use crate::{DefMap, Resolution};

/// Name resolution output used by type checking and HIR lowering.
#[derive(Clone, Debug)]
pub struct ResolvedCrate {
    pub def_map: DefMap,
    pub expr_bindings: FxHashMap<ExprId, NameBinding>,
    pub item_def_ids: Vec<(ItemId, DefId)>,
    pub root: DefId,
    pub builtins: crate::BuiltinDefs,
}

impl ResolvedCrate {
    pub fn from_resolution(resolution: Resolution, module: &Module, arena: &Arena) -> Self {
        let root = resolution.def_map.root_def();
        let mut item_def_ids = Vec::new();
        for &item_id in &module.items {
            let item = arena.item(item_id);
            let name = item_name(&item.kind);
            if let Some(name) = name {
                if let Some(def) = resolution.def_map.lookup(root, &name) {
                    item_def_ids.push((item_id, def));
                }
            }
        }
        Self {
            def_map: resolution.def_map,
            expr_bindings: resolution.expr_bindings,
            item_def_ids,
            root,
            builtins: resolution.builtins,
        }
    }
}

fn item_name(kind: &ItemKind) -> Option<SmolStr> {
    match kind {
        ItemKind::Function { name, .. }
        | ItemKind::Struct { name, .. }
        | ItemKind::Enum { name, .. }
        | ItemKind::Trait { name, .. }
        | ItemKind::Class { name, .. }
        | ItemKind::Const { name, .. } => Some(name.clone()),
        ItemKind::Module { name, .. } => Some(name.clone()),
        _ => None,
    }
}

/// Resolve a simple or qualified path to a definition.
pub fn resolve_path(path: &Path, resolved: &ResolvedCrate) -> Option<DefId> {
    if path.segments.is_empty() {
        return None;
    }
    let first = &path.segments[0].ident;
    let mut current = resolved.def_map.lookup(resolved.root, first)?;
    for seg in path.segments.iter().skip(1) {
        current = resolved.def_map.lookup(current, &seg.ident)?;
    }
    Some(current)
}
