use crate::place::{Place, Projection};
use crate::value::Value;

#[derive(Clone, Debug, Default)]
pub struct Frame {
    pub slots: Vec<Option<Value>>,
}

impl Frame {
    pub fn new(local_count: usize) -> Self {
        Self {
            slots: vec![None; local_count],
        }
    }

    pub fn set(&mut self, place: &Place, value: Value) {
        if place.projection.is_empty() {
            self.slots[place.local.index()] = Some(value);
            return;
        }
        let base = self.slots[place.local.index()]
            .take()
            .unwrap_or(Value::Unit);
        self.slots[place.local.index()] = Some(project_set(base, &place.projection, value));
    }

    pub fn get(&self, place: &Place) -> Value {
        let base = self
            .slots
            .get(place.local.index())
            .and_then(|s| s.as_ref())
            .cloned()
            .unwrap_or(Value::Unit);
        project_get(base, &place.projection)
    }
}

fn project_get(mut val: Value, projs: &[Projection]) -> Value {
    for proj in projs {
        val = match proj {
            Projection::Field(i) => match val {
                Value::Struct { fields, .. } | Value::Tuple(fields) => {
                    fields.get(*i as usize).cloned().unwrap_or(Value::Unit)
                }
                _ => Value::Unit,
            },
            Projection::Downcast(v) => match val {
                Value::Enum {
                    variant, payload, ..
                } if variant == *v => payload.map(|p| *p).unwrap_or(Value::Unit),
                _ => Value::Unit,
            },
            Projection::Deref | Projection::Index(_) => val,
        };
    }
    val
}

fn project_set(mut val: Value, projs: &[Projection], new_val: Value) -> Value {
    if projs.is_empty() {
        return new_val;
    }
    match (&projs[0], &mut val) {
        (Projection::Field(i), Value::Struct { fields, .. })
        | (Projection::Field(i), Value::Tuple(fields)) => {
            if let Some(slot) = fields.get_mut(*i as usize) {
                *slot = project_set(slot.clone(), &projs[1..], new_val);
            }
        }
        (
            Projection::Downcast(v),
            Value::Enum {
                variant, payload, ..
            },
        ) if *variant == *v => {
            *payload = Some(Box::new(project_set(
                payload.take().map(|p| *p).unwrap_or(Value::Unit),
                &projs[1..],
                new_val,
            )));
        }
        _ => {}
    }
    val
}
