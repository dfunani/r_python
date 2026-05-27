use indexmap::IndexMap;
use rpython_ids::{DefId, ImplId, TypeId};
use rpython_types::Subst;

/// One `impl` block entry for method dispatch.
#[derive(Clone, Debug)]
pub struct ImplEntry {
    pub impl_id: ImplId,
    pub def_id: DefId,
    pub self_ty: TypeId,
    pub interface_ref: Option<DefId>,
    pub methods: IndexMap<smol_str::SmolStr, DefId>,
}

/// Table of impl blocks indexed by type and interface.
#[derive(Clone, Debug, Default)]
pub struct ImplTable {
    entries: Vec<ImplEntry>,
}

impl ImplTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, entry: ImplEntry) {
        self.entries.push(entry);
    }

    pub fn find_method(&self, self_ty: TypeId, method: &str) -> Option<DefId> {
        for entry in &self.entries {
            if entry.self_ty == self_ty {
                if let Some(def) = entry.methods.get(method) {
                    return Some(*def);
                }
            }
        }
        None
    }

    pub fn entries(&self) -> &[ImplEntry] {
        &self.entries
    }
}

/// Monomorphized function instance.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MonoInstance {
    pub def_id: DefId,
    pub subst: Subst,
}

/// Audit log entry for fulfilled interface obligations (v2 stub).
#[derive(Clone, Debug)]
pub struct FulfilledObligation {
    pub interface_def: DefId,
    pub self_ty: TypeId,
}
