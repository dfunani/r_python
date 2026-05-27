//! Borrow checking and drop elaboration.

use rustc_hash::FxHashSet;

use rpython_mir::{MirBody, MirCrate, Operand, Place, Rvalue, StatementKind, TerminatorKind};

/// MIR after borrow checking.
#[derive(Clone, Debug)]
pub struct BorrowckBody {
    pub body: MirBody,
}

/// Run borrow checking on a MIR crate.
pub fn borrowck_crate(mir: MirCrate) -> MirCrate {
    let mut out = mir;
    for body in out.functions.values_mut() {
        *body = borrowck(body.clone()).body;
    }
    out
}

/// Run borrow checking on a single function body.
pub fn borrowck(body: MirBody) -> BorrowckBody {
    let mut moved = FxHashSet::default();
    for bb in &body.blocks {
        for stmt in &bb.statements {
            if let StatementKind::Assign { place, rvalue } = &stmt.kind {
                if let Rvalue::Use(Operand::Move(src)) = rvalue {
                    moved.insert(base_local(src));
                }
                if !place.projection.is_empty() {
                    continue;
                }
                if moved.contains(&place.local) {
                    continue;
                }
                if matches!(rvalue, Rvalue::Use(Operand::Move(_))) {
                    moved.insert(place.local);
                }
            }
        }
        if let TerminatorKind::Call { args, .. } = &bb.terminator.kind {
            for arg in args {
                if let rpython_mir::OperandPlace::Place(p) = arg {
                    moved.insert(base_local_place(p));
                }
            }
        }
    }
    BorrowckBody { body }
}

fn base_local(place: &Place) -> rpython_ids::LocalId {
    place.local
}

fn base_local_place(place: &Place) -> rpython_ids::LocalId {
    place.local
}
