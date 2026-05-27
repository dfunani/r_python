use crate::TypeDatabase;
use rpython_ids::{DefId, TypeId};

/// Substitution mapping generic parameters to concrete types.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Subst {
    pub args: Vec<TypeId>,
}

impl Subst {
    pub fn empty() -> Self {
        Self { args: Vec::new() }
    }

    pub fn from_args(args: Vec<TypeId>) -> Self {
        Self { args }
    }

    pub fn identity(n: usize, db: &mut TypeDatabase) -> Self {
        let mut args = Vec::with_capacity(n);
        for i in 0..n {
            args.push(db.generic_param(i as u32));
        }
        Self { args }
    }

    pub fn apply(&self, db: &mut TypeDatabase, ty: TypeId) -> TypeId {
        db.substitute(ty, self)
    }

    pub fn apply_def(&self, def: DefId) -> DefId {
        def
    }
}
