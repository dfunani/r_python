use rpython_ids::DefId;

use crate::place::Place;

/// MIR operand.
#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(ConstValue),
}

/// MIR constant value.
#[derive(Clone, Debug, PartialEq)]
pub enum ConstValue {
    Int(i64),
    Bool(bool),
    Float(f64),
    Str(String),
    Unit,
    ZeroSized,
}

/// Function reference for calls.
#[derive(Clone, Debug, PartialEq)]
pub enum FnOperand {
    Def(DefId),
    Place(Place),
}
