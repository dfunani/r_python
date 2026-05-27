use rpython_ids::LocalId;
use rpython_span::Span;

use crate::place::Place;
use crate::rvalue::Rvalue;

#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StatementKind {
    Assign { place: Place, rvalue: Rvalue },
    StorageLive(LocalId),
    StorageDead(LocalId),
    Deinit(Place),
    Nop,
}
