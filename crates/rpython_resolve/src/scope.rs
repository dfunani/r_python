use indexmap::IndexMap;
use rpython_ids::DefId;
use smol_str::SmolStr;

/// Kind of lexical scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Root,
    Module,
    Function,
    Block,
    Impl,
    Interface,
    Class,
}

/// One lexical scope frame.
#[derive(Clone, Debug)]
pub struct Scope {
    pub kind: ScopeKind,
    pub parent: Option<DefId>,
    pub owner: DefId,
    pub bindings: IndexMap<SmolStr, DefId>,
}

impl Scope {
    pub fn new(kind: ScopeKind, owner: DefId, parent: Option<DefId>) -> Self {
        Self {
            kind,
            parent,
            owner,
            bindings: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, name: SmolStr, def: DefId) -> Option<DefId> {
        self.bindings.insert(name, def)
    }

    pub fn get(&self, name: &str) -> Option<DefId> {
        self.bindings.get(name).copied()
    }
}
