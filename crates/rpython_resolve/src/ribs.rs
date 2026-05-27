use crate::scope::{Scope, ScopeKind};
use rpython_ids::DefId;
use smol_str::SmolStr;

/// Stack of lexical scopes (ribs).
#[derive(Clone, Debug, Default)]
pub struct RibStack {
    scopes: Vec<Scope>,
}

impl RibStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, kind: ScopeKind, owner: DefId, parent: Option<DefId>) {
        self.scopes.push(Scope::new(kind, owner, parent));
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn current_owner(&self) -> DefId {
        self.scopes
            .last()
            .map(|s| s.owner)
            .unwrap_or(DefId(0))
    }

    pub fn current(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("rib stack empty")
    }

    pub fn define(&mut self, name: SmolStr, def: DefId) -> Option<DefId> {
        self.current().insert(name, def)
    }

    /// Resolve `name` walking from innermost to outermost scope.
    pub fn resolve(&self, name: &str) -> Option<DefId> {
        for scope in self.scopes.iter().rev() {
            if let Some(def) = scope.get(name) {
                return Some(def);
            }
            if matches!(
                scope.kind,
                ScopeKind::Function | ScopeKind::Impl | ScopeKind::Trait | ScopeKind::Class
            ) {
                break;
            }
        }
        None
    }

    pub fn depth(&self) -> usize {
        self.scopes.len()
    }
}
