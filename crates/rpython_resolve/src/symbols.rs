use rpython_ids::{DefId, SymbolId};

/// Resolved binding for a name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub def: DefId,
}

/// Name resolution result attached to a path expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameBinding {
    pub symbol: SymbolId,
    pub def: DefId,
}

impl NameBinding {
    pub fn new(def: DefId) -> Self {
        Self {
            symbol: SymbolId(def.0),
            def,
        }
    }
}
