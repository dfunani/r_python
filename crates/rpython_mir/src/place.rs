use rpython_ids::LocalId;

/// MIR place (lvalue).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Place {
    pub local: LocalId,
    pub projection: Vec<Projection>,
}

/// Projections on a MIR place.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Projection {
    Field(u32),
    Index(LocalId),
    Deref,
    Downcast(u32),
}

impl Place {
    pub fn local(local: LocalId) -> Self {
        Self {
            local,
            projection: Vec::new(),
        }
    }

    pub fn return_place() -> Self {
        Self::local(LocalId::from_usize(0))
    }
}
