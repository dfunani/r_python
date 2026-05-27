use crate::rvalue::{BinOp, UnaryOp};
use crate::value::Value;

pub fn unary_op(op: UnaryOp, val: Value) -> Value {
    match op {
        UnaryOp::Not => Value::Bool(!val.as_bool().unwrap_or(false)),
        UnaryOp::Neg => Value::Int(-val.as_int().unwrap_or(0)),
    }
}

pub fn binary_op(op: BinOp, left: Value, right: Value) -> Value {
    match op {
        BinOp::Add => Value::Int(left.as_int().unwrap_or(0) + right.as_int().unwrap_or(0)),
        BinOp::Sub => Value::Int(left.as_int().unwrap_or(0) - right.as_int().unwrap_or(0)),
        BinOp::Mul => Value::Int(left.as_int().unwrap_or(0) * right.as_int().unwrap_or(0)),
        BinOp::Div => Value::Int(left.as_int().unwrap_or(0) / right.as_int().unwrap_or(1)),
        BinOp::Rem => Value::Int(left.as_int().unwrap_or(0) % right.as_int().unwrap_or(1)),
        BinOp::Eq => Value::Bool(left.as_int() == right.as_int()),
        BinOp::Ne => Value::Bool(left.as_int() != right.as_int()),
        BinOp::Lt => Value::Bool(left.as_int().unwrap_or(0) < right.as_int().unwrap_or(0)),
        BinOp::Le => Value::Bool(left.as_int().unwrap_or(0) <= right.as_int().unwrap_or(0)),
        BinOp::Gt => Value::Bool(left.as_int().unwrap_or(0) > right.as_int().unwrap_or(0)),
        BinOp::Ge => Value::Bool(left.as_int().unwrap_or(0) >= right.as_int().unwrap_or(0)),
        BinOp::And => Value::Bool(left.as_bool().unwrap_or(false) && right.as_bool().unwrap_or(false)),
        BinOp::Or => Value::Bool(left.as_bool().unwrap_or(false) || right.as_bool().unwrap_or(false)),
    }
}
