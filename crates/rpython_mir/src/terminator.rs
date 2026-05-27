use rpython_ids::BlockId;
use rpython_span::Span;

use crate::operand::FnOperand;
use crate::place::Place;

#[derive(Clone, Debug, PartialEq)]
pub struct Terminator {
    pub kind: TerminatorKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TerminatorKind {
    Goto {
        target: BlockId,
    },
    SwitchInt {
        discr: OperandPlace,
        targets: Vec<(u128, BlockId)>,
        otherwise: BlockId,
    },
    Return,
    Unreachable,
    Drop {
        place: Place,
        target: BlockId,
        unwind: Option<BlockId>,
    },
    Call {
        func: FnOperand,
        args: Vec<OperandPlace>,
        destination: Option<Place>,
        target: BlockId,
        unwind: Option<BlockId>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum OperandPlace {
    Place(Place),
    ConstInt(i64),
    ConstBool(bool),
    ConstStr(String),
}
