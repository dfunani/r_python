//! Mid-level IR (MIR) for rPython.

mod body;
pub mod interp;
mod operand;
mod place;
mod pretty;
mod rvalue;
mod statement;
mod terminator;
mod value;
mod visit;

pub use body::{BasicBlock, LocalDecl, MirBody, MirCrate, SourceScope};
pub use operand::{ConstValue, FnOperand, Operand};
pub use place::{Place, Projection};
pub use pretty::{pretty_print_mir, MirPrinter};
pub use rvalue::{AggregateKind, BinOp, CastKind, Rvalue, UnaryOp};
pub use statement::{Statement, StatementKind};
pub use terminator::{OperandPlace, Terminator, TerminatorKind};
pub use value::Value;
pub use visit::{visit_body, MirVisitor};

pub use rpython_ids::{BlockId, DefId, LocalId, MirFuncId};

pub fn format_mir_crate(crate_: &MirCrate) -> String {
    let mut out = String::new();
    for body in crate_.functions.values() {
        out.push_str(&pretty_print_mir(body));
        out.push('\n');
    }
    out
}
