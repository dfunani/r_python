//! Borrow checking and drop elaboration (v1: pass-through).

use rpython_mir::{MirBody, MirCrate};

/// MIR after borrow checking.
#[derive(Clone, Debug)]
pub struct BorrowckBody {
    pub body: MirBody,
}

/// Run borrow checking on a MIR crate (v1: no-op).
pub fn borrowck_crate(mir: MirCrate) -> MirCrate {
    mir
}

/// Run borrow checking on a single function body.
pub fn borrowck(body: MirBody) -> BorrowckBody {
    BorrowckBody { body }
}
