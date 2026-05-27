mod memory;
mod ops;

use rpython_ids::{BlockId, MirFuncId};

use crate::body::{MirBody, MirCrate};
use crate::operand::{ConstValue, FnOperand, Operand};
use crate::place::Place;
use crate::rvalue::{AggregateKind, Rvalue};
use crate::statement::{Statement, StatementKind};
use crate::terminator::{OperandPlace, TerminatorKind};
use crate::value::Value;

pub use memory::Frame;

/// Run `main` (or the first function) in a crate via the MIR interpreter.
pub fn interpret_crate(crate_: &MirCrate) -> Result<(), String> {
    let main_def = crate_
        .functions
        .values()
        .find(|f| f.name.as_str() == "main")
        .map(|f| f.def_id);
    let func_id = if let Some(def) = main_def {
        *crate_.def_to_func.get(&def).ok_or("main not in def_to_func")?
    } else {
        *crate_
            .def_to_func
            .values()
            .next()
            .ok_or("no functions in crate")?
    };
    let _ = interpret(crate_, func_id, vec![]);
    Ok(())
}

/// Interpret a function in a crate (supports calls).
pub fn interpret(crate_: &MirCrate, func: MirFuncId, args: Vec<Value>) -> Value {
    let body = crate_
        .functions
        .get(&func)
        .expect("unknown MirFuncId");
    interpret_body(crate_, body, args)
}

/// Interpret one MIR body; `crate_` resolves `Call` terminators.
pub fn interpret_body(crate_: &MirCrate, body: &MirBody, args: Vec<Value>) -> Value {
    let mut frame = Frame::new(body.locals.len());
    for (i, arg) in args.iter().enumerate() {
        frame.set(&Place::local(body.arg_local(i)), arg.clone());
    }

    let mut block = BlockId::from_usize(0);
    loop {
        let bb = &body.blocks[block.index()];
        for stmt in &bb.statements {
            eval_statement(&mut frame, stmt);
        }
        match &bb.terminator.kind {
            TerminatorKind::Goto { target } => block = *target,
            TerminatorKind::SwitchInt {
                discr,
                targets,
                otherwise,
            } => {
                let d = eval_operand_place(&frame, discr).discriminant_u128();
                block = targets
                    .iter()
                    .find(|(v, _)| *v == d)
                    .map(|(_, b)| *b)
                    .unwrap_or(*otherwise);
            }
            TerminatorKind::Return => return frame.get(&Place::return_place()),
            TerminatorKind::Call {
                func,
                args: call_args,
                destination,
                target,
                ..
            } => {
                if let Some(val) = eval_call(crate_, func, call_args, &frame) {
                    if let Some(dest) = destination {
                        frame.set(dest, val);
                    }
                }
                block = *target;
            }
            TerminatorKind::Drop { target, .. } => block = *target,
            TerminatorKind::Unreachable => panic!("unreachable MIR"),
        }
    }
}

fn eval_call(
    crate_: &MirCrate,
    func: &FnOperand,
    args: &[OperandPlace],
    frame: &Frame,
) -> Option<Value> {
    let FnOperand::Def(def) = func else {
        return None;
    };
    if let Some(mir_id) = crate_.def_to_func.get(def) {
        let arg_vals: Vec<Value> = args
            .iter()
            .map(|a| eval_operand_place(frame, a))
            .collect();
        return Some(interpret(crate_, *mir_id, arg_vals));
    }
    // Builtins (not lowered to MIR bodies): `print` for now.
    if let Some(arg) = args.first() {
        match eval_operand_place(frame, arg) {
            Value::String(s) => {
                println!("{s}");
            }
            Value::Int(n) => {
                println!("{n}");
            }
            Value::Bool(b) => {
                println!("{b}");
            }
            Value::Unit => {
                println!();
            }
            other => {
                println!("{other:?}");
            }
        }
    } else {
        println!();
    }
    Some(Value::Unit)
}

fn eval_statement(frame: &mut Frame, stmt: &Statement) {
    if let StatementKind::Assign { place, rvalue } = &stmt.kind {
        frame.set(place, eval_rvalue(frame, rvalue));
    }
}

fn eval_rvalue(frame: &Frame, rv: &Rvalue) -> Value {
    match rv {
        Rvalue::Use(op) => eval_operand(frame, op),
        Rvalue::UnaryOp { op, operand } => ops::unary_op(*op, eval_operand(frame, operand)),
        Rvalue::BinaryOp { op, left, right } => ops::binary_op(
            *op,
            eval_operand(frame, left),
            eval_operand(frame, right),
        ),
        Rvalue::Aggregate { kind, ops } => {
            let vals: Vec<Value> = ops.iter().map(|o| eval_operand(frame, o)).collect();
            match kind {
                AggregateKind::Tuple => Value::Tuple(vals),
                AggregateKind::Struct(def) => Value::Struct {
                    def: *def,
                    fields: vals,
                },
                AggregateKind::Enum(def, variant) => Value::Enum {
                    def: *def,
                    variant: *variant,
                    payload: vals.into_iter().next().map(Box::new),
                },
                AggregateKind::Array(_) => Value::Tuple(vals),
            }
        }
        Rvalue::Ref { place, .. } => frame.get(place),
        Rvalue::Len(p) => Value::Int(match frame.get(p) {
            Value::Tuple(v) => v.len() as i64,
            Value::String(v) => v.len() as i64,
            _ => 0,
        }),
        Rvalue::Discriminant(p) => Value::Int(frame.get(p).discriminant_u128() as i64),
        Rvalue::Cast { operand, .. } => eval_operand(frame, operand),
    }
}

fn eval_operand(frame: &Frame, op: &Operand) -> Value {
    match op {
        Operand::Copy(p) | Operand::Move(p) => frame.get(p),
        Operand::Constant(c) => match c {
            ConstValue::Int(n) => Value::Int(*n),
            ConstValue::Bool(b) => Value::Bool(*b),
            ConstValue::Float(f) => Value::Float(*f),
            ConstValue::Str(s) => Value::String(s.clone()),
            ConstValue::Unit | ConstValue::ZeroSized => Value::Unit,
        },
    }
}

fn eval_operand_place(frame: &Frame, op: &OperandPlace) -> Value {
    match op {
        OperandPlace::Place(p) => frame.get(p),
        OperandPlace::ConstInt(n) => Value::Int(*n),
        OperandPlace::ConstBool(b) => Value::Bool(*b),
        OperandPlace::ConstStr(s) => Value::String(s.clone()),
    }
}
