use rpython_ids::{DefId, TypeId};
use rpython_types::{Mutability, RegionId};

use crate::operand::Operand;
use crate::place::Place;

/// MIR rvalue.
#[derive(Clone, Debug, PartialEq)]
pub enum Rvalue {
    Use(Operand),
    UnaryOp {
        op: UnaryOp,
        operand: Operand,
    },
    BinaryOp {
        op: BinOp,
        left: Operand,
        right: Operand,
    },
    Aggregate {
        kind: AggregateKind,
        ops: Vec<Operand>,
    },
    Ref {
        region: RegionId,
        mutability: Mutability,
        place: Place,
    },
    Len(Place),
    Cast {
        kind: CastKind,
        operand: Operand,
        ty: TypeId,
    },
    Discriminant(Place),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AggregateKind {
    Tuple,
    Struct(DefId),
    Enum(DefId, u32),
    Array(TypeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastKind {
    IntToInt,
    FloatToInt,
    IntToFloat,
}
