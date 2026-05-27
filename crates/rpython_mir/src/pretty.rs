use std::fmt::{self, Write};

use rpython_ids::BlockId;

use crate::body::{BasicBlock, MirBody};
use crate::operand::{ConstValue, Operand};
use crate::place::{Place, Projection};
use crate::rvalue::Rvalue;
use crate::statement::{Statement, StatementKind};
use crate::terminator::{OperandPlace, Terminator, TerminatorKind};

pub struct MirPrinter<'a> {
    body: &'a MirBody,
    out: String,
}

impl<'a> MirPrinter<'a> {
    pub fn new(body: &'a MirBody) -> Self {
        Self {
            body,
            out: String::new(),
        }
    }

    pub fn print(mut self) -> String {
        let _ = writeln!(self.out, "fn {}:", self.body.name);
        for (i, local) in self.body.locals.iter().enumerate() {
            let _ = writeln!(
                self.out,
                "  _{i}: ty={:?} mut={:?}",
                local.ty, local.mutability
            );
        }
        for (i, bb) in self.body.blocks.iter().enumerate() {
            self.print_block(BlockId::from_usize(i), bb);
        }
        self.out
    }

    fn print_block(&mut self, id: BlockId, bb: &BasicBlock) {
        let _ = writeln!(self.out, "  bb{}:", id.index());
        for stmt in &bb.statements {
            self.print_stmt(stmt);
        }
        self.print_term(&bb.terminator);
    }

    fn print_stmt(&mut self, stmt: &Statement) {
        match &stmt.kind {
            StatementKind::Assign { place, rvalue } => {
                let _ = writeln!(self.out, "      {place} = {}", self.fmt_rvalue(rvalue));
            }
            StatementKind::StorageLive(l) => {
                let _ = writeln!(self.out, "      StorageLive(_{})", l.index());
            }
            StatementKind::StorageDead(l) => {
                let _ = writeln!(self.out, "      StorageDead(_{})", l.index());
            }
            StatementKind::Deinit(p) => {
                let _ = writeln!(self.out, "      Deinit({p})");
            }
            StatementKind::Nop => {
                let _ = writeln!(self.out, "      Nop");
            }
        }
    }

    fn print_term(&mut self, term: &Terminator) {
        match &term.kind {
            TerminatorKind::Goto { target } => {
                let _ = writeln!(self.out, "      goto -> bb{}", target.index());
            }
            TerminatorKind::SwitchInt {
                discr,
                targets,
                otherwise,
            } => {
                let _ = writeln!(
                    self.out,
                    "      switchInt({}) -> {:?}, otherwise bb{}",
                    self.fmt_op_place(discr),
                    targets
                        .iter()
                        .map(|(v, b)| format!("{v}: bb{}", b.index()))
                        .collect::<Vec<_>>(),
                    otherwise.index()
                );
            }
            TerminatorKind::Return => {
                let _ = writeln!(self.out, "      return");
            }
            TerminatorKind::Unreachable => {
                let _ = writeln!(self.out, "      unreachable");
            }
            TerminatorKind::Drop { place, target, .. } => {
                let _ = writeln!(self.out, "      drop({place}) -> bb{}", target.index());
            }
            TerminatorKind::Call {
                func,
                args,
                destination,
                target,
                ..
            } => {
                let _ = writeln!(
                    self.out,
                    "      call {func:?}({}) -> {destination:?} -> bb{}",
                    args.iter()
                        .map(|a| self.fmt_op_place(a))
                        .collect::<Vec<_>>()
                        .join(", "),
                    target.index()
                );
            }
        }
    }

    fn fmt_rvalue(&self, rv: &Rvalue) -> String {
        match rv {
            Rvalue::Use(op) => self.fmt_operand(op),
            Rvalue::UnaryOp { op, operand } => format!("{op:?} {}", self.fmt_operand(operand)),
            Rvalue::BinaryOp { op, left, right } => format!(
                "{} {op:?} {}",
                self.fmt_operand(left),
                self.fmt_operand(right)
            ),
            Rvalue::Aggregate { kind, ops } => format!(
                "Aggregate({kind:?}, [{}])",
                ops.iter()
                    .map(|o| self.fmt_operand(o))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Rvalue::Ref {
                mutability, place, ..
            } => format!("&{mutability:?} {place}"),
            Rvalue::Len(p) => format!("len({p})"),
            Rvalue::Cast { operand, .. } => format!("cast({})", self.fmt_operand(operand)),
            Rvalue::Discriminant(p) => format!("discriminant({p})"),
        }
    }

    fn fmt_operand(&self, op: &Operand) -> String {
        match op {
            Operand::Copy(p) | Operand::Move(p) => format!("{p}"),
            Operand::Constant(c) => self.fmt_const(c),
        }
    }

    fn fmt_const(&self, c: &ConstValue) -> String {
        match c {
            ConstValue::Int(n) => format!("const {n}"),
            ConstValue::Bool(b) => format!("const {b}"),
            ConstValue::Float(f) => format!("const {f}"),
            ConstValue::Unit => "const ()".into(),
            ConstValue::ZeroSized => "const ZST".into(),
            ConstValue::Str(s) => format!("const \"{s}\""),
        }
    }

    fn fmt_op_place(&self, op: &OperandPlace) -> String {
        match op {
            OperandPlace::Place(p) => format!("{p}"),
            OperandPlace::ConstInt(n) => format!("{n}"),
            OperandPlace::ConstBool(b) => format!("{b}"),
            OperandPlace::ConstStr(s) => format!("\"{s}\""),
        }
    }
}

pub fn pretty_print_mir(body: &MirBody) -> String {
    MirPrinter::new(body).print()
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_{}", self.local.index())?;
        for proj in &self.projection {
            match proj {
                Projection::Field(i) => write!(f, ".f{i}")?,
                Projection::Index(l) => write!(f, "[_{}]", l.index())?,
                Projection::Deref => write!(f, ".*")?,
                Projection::Downcast(v) => write!(f, ":{v}")?,
            }
        }
        Ok(())
    }
}
