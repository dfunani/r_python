use rpython_ids::DefId;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Tuple(Vec<Value>),
    Struct {
        def: DefId,
        fields: Vec<Value>,
    },
    Enum {
        def: DefId,
        variant: u32,
        payload: Option<Box<Value>>,
    },
}

impl Value {
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn discriminant_u128(&self) -> u128 {
        match self {
            Value::Bool(false) => 0,
            Value::Bool(true) => 1,
            Value::Int(n) => *n as u128,
            Value::Enum { variant, .. } => *variant as u128,
            _ => 0,
        }
    }
}
