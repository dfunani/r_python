use rpython_ids::LocalId;
use rpython_types::Mutability;

use crate::ids::HirExprId;

/// A place in HIR (lvalue).
#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub local: LocalId,
    pub projection: Vec<Projection>,
}

/// Projections on a place.
#[derive(Clone, Debug, PartialEq)]
pub enum Projection {
    Field(u32),
    Index(HirExprId),
    Deref,
}

/// Operand in HIR rvalues.
#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(HirConst),
}

/// HIR constant value.
#[derive(Clone, Debug, PartialEq)]
pub enum HirConst {
    Int(i64),
    Bool(bool),
    Float(f64),
    Str(smol_str::SmolStr),
    Unit,
}

/// HIR rvalue.
#[derive(Clone, Debug, PartialEq)]
pub enum Rvalue {
    Use(Operand),
    UnaryOp {
        op: UnaryOp,
        operand: Operand,
    },
    BinaryOp {
        op: BinaryOp,
        left: Operand,
        right: Operand,
    },
    Aggregate(AggregateKind, Vec<Operand>),
    Ref {
        mutability: Mutability,
        place: Place,
    },
    Len(Place),
    Discriminant(Place),
}

/// Unary operators in HIR.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

/// Binary operators in HIR.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    Mod,
}

/// Aggregate kinds.
#[derive(Clone, Debug, PartialEq)]
pub enum AggregateKind {
    Tuple,
    Struct(rpython_ids::DefId),
    Enum(rpython_ids::DefId, u32),
}

impl Place {
    pub fn local(local: LocalId) -> Self {
        Self {
            local,
            projection: Vec::new(),
        }
    }
}
