use crate::Subst;
use rpython_ids::TraitId;

/// Reference to a trait with substituted type arguments.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitRef {
    pub trait_id: TraitId,
    pub subst: Subst,
}
