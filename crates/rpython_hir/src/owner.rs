use indexmap::IndexMap;
use rpython_ids::DefId;

use crate::stmt::HirBody;

/// HIR owner (one per definition).
#[derive(Clone, Debug)]
pub enum HirOwnerKind {
    Function(HirBody),
}

/// HIR owner node.
#[derive(Clone, Debug)]
pub struct HirOwner {
    pub def_id: DefId,
    pub kind: HirOwnerKind,
}

/// Whole-crate HIR.
#[derive(Clone, Debug, Default)]
pub struct HirCrate {
    pub owners: IndexMap<DefId, HirOwner>,
}
