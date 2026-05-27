use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A syntax-level literal value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(SmolStr),
    Bytes(Vec<u8>),
    Bool(bool),
    Char(char),
    None,
}
