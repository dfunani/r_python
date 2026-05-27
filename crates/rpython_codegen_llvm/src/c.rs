//! Emit C source from MIR.

use std::fmt::Write;

use rpython_mir::{
    BasicBlock, BinOp, ConstValue, FnOperand, MirBody, MirCrate, Operand, OperandPlace, Rvalue,
    StatementKind, TerminatorKind, UnaryOp,
};
use rpython_resolve::Resolution;
use rpython_typeck::TypedCrate;
use rpython_types::TyKind;

#[derive(Clone, Debug)]
pub struct COutput {
    pub source: String,
}

pub fn compile_crate_to_c(mir: &MirCrate, typed: &TypedCrate, resolution: &Resolution) -> COutput {
    let mut emitter = CEmitter {
        out: String::new(),
        typed,
        resolution,
        strings: Vec::new(),
    };
    emitter.emit_preamble();
    for body in mir.functions.values() {
        emitter.collect_strings(body);
    }
    emitter.emit_string_pool();
    for body in mir.functions.values() {
        emitter.emit_function(body);
    }
    emitter.emit_main_wrapper(mir);
    COutput {
        source: emitter.out,
    }
}

struct CEmitter<'a> {
    out: String,
    typed: &'a TypedCrate,
    resolution: &'a Resolution,
    strings: Vec<String>,
}

impl<'a> CEmitter<'a> {
    fn emit_preamble(&mut self) {
        self.out.push_str(
            "#include <stdint.h>\n#include <stdio.h>\n\n\
             void rpy_rt_init(void);\n\
             void rpy_panic(const char *, int64_t);\n\
             void rpy_print_str(const char *, int64_t);\n\
             void rpy_print_int(int64_t);\n\
             void rpy_print_bool(int8_t);\n\
             void rpy_print_newline(void);\n\n\
             typedef struct { const char *ptr; int64_t len; } rpy_str;\n\n",
        );
    }

    fn intern_string(&mut self, s: &str) -> usize {
        if let Some(i) = self.strings.iter().position(|x| x == s) {
            return i;
        }
        let i = self.strings.len();
        self.strings.push(s.to_string());
        i
    }

    fn mangle(&self, name: &str) -> String {
        format!("_rpy__{name}")
    }

    fn collect_strings(&mut self, body: &MirBody) {
        for bb in &body.blocks {
            for stmt in &bb.statements {
                if let StatementKind::Assign { rvalue, .. } = &stmt.kind {
                    self.collect_rvalue_strings(rvalue);
                }
            }
            match &bb.terminator.kind {
                TerminatorKind::Call { args, .. } => {
                    for arg in args {
                        if let OperandPlace::ConstStr(s) = arg {
                            self.intern_string(s);
                        }
                    }
                }
                TerminatorKind::SwitchInt {
                    discr: OperandPlace::ConstStr(s),
                    ..
                } => {
                    self.intern_string(s);
                }
                _ => {}
            }
        }
    }

    fn collect_operand_strings(&mut self, op: &Operand) {
        if let Operand::Constant(ConstValue::Str(s)) = op {
            self.intern_string(s);
        }
    }

    fn collect_rvalue_strings(&mut self, rv: &Rvalue) {
        match rv {
            Rvalue::Use(op) => self.collect_operand_strings(op),
            Rvalue::UnaryOp { operand, .. } => self.collect_operand_strings(operand),
            Rvalue::BinaryOp { left, right, .. } => {
                self.collect_operand_strings(left);
                self.collect_operand_strings(right);
            }
            _ => {}
        }
    }

    fn emit_function(&mut self, body: &MirBody) {
        let ret = self.c_type(body.return_ty);
        let name = self.mangle(body.name.as_str());
        writeln!(self.out, "{ret} {name}(void) {{").ok();
        for (i, bb) in body.blocks.iter().enumerate() {
            writeln!(self.out, "bb{i}: {{").ok();
            self.emit_block(bb, body);
            writeln!(self.out, "}}").ok();
        }
        writeln!(self.out, "}}\n").ok();
    }

    fn emit_block(&mut self, bb: &BasicBlock, body: &MirBody) {
        for stmt in &bb.statements {
            if let StatementKind::Assign { place, rvalue } = &stmt.kind {
                let lhs = format!("_{}", place.local.index());
                let rhs = self.emit_rvalue(rvalue, body);
                let c_ty = self.c_type(body.local_ty(place.local));
                writeln!(self.out, "  {c_ty} {lhs} = {rhs};").ok();
            }
        }
        match &bb.terminator.kind {
            TerminatorKind::Goto { target } => {
                writeln!(self.out, "  goto bb{};", target.index()).ok();
            }
            TerminatorKind::Return => {
                writeln!(self.out, "  return (int)_{};", 0).ok();
            }
            TerminatorKind::SwitchInt {
                discr,
                targets,
                otherwise,
            } => {
                let d = self.emit_operand_place(discr);
                for (val, tgt) in targets {
                    writeln!(self.out, "  if ({d} == {val}) goto bb{};", tgt.index()).ok();
                }
                writeln!(self.out, "  goto bb{};", otherwise.index()).ok();
            }
            TerminatorKind::Call {
                func,
                args,
                destination: _,
                target,
                ..
            } => {
                self.emit_call(func, args, body);
                writeln!(self.out, "  goto bb{};", target.index()).ok();
            }
            TerminatorKind::Unreachable => {
                writeln!(self.out, "  rpy_panic(\"unreachable\", 11);").ok();
            }
            TerminatorKind::Drop { target, .. } => {
                writeln!(self.out, "  goto bb{};", target.index()).ok();
            }
        }
    }

    fn emit_call(&mut self, func: &FnOperand, args: &[OperandPlace], body: &MirBody) {
        let FnOperand::Def(def) = func else {
            return;
        };
        if *def == self.resolution.builtins.print {
            if let Some(arg) = args.first() {
                let op = self.emit_operand_place(arg);
                let ty = self.operand_place_ty(arg, body);
                match ty {
                    TyKind::Str => {
                        writeln!(
                            self.out,
                            "  rpy_print_str({op}.ptr, {op}.len); rpy_print_newline();"
                        )
                        .ok();
                    }
                    TyKind::Bool => {
                        writeln!(
                            self.out,
                            "  rpy_print_bool((int8_t){op}); rpy_print_newline();"
                        )
                        .ok();
                    }
                    _ => {
                        writeln!(self.out, "  rpy_print_int({op}); rpy_print_newline();").ok();
                    }
                }
            } else {
                writeln!(self.out, "  rpy_print_newline();").ok();
            }
            return;
        }
        let name = self
            .resolution
            .def_map
            .name(*def)
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".into());
        writeln!(self.out, "  {}();", self.mangle(&name)).ok();
    }

    fn emit_rvalue(&mut self, rv: &Rvalue, body: &MirBody) -> String {
        match rv {
            Rvalue::Use(op) => self.emit_operand(op, body),
            Rvalue::UnaryOp { op, operand } => {
                let o = self.emit_operand(operand, body);
                match op {
                    UnaryOp::Not => format!("(!{o})"),
                    UnaryOp::Neg => format!("(-{o})"),
                }
            }
            Rvalue::BinaryOp { op, left, right } => {
                let l = self.emit_operand(left, body);
                let r = self.emit_operand(right, body);
                let sym = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Rem => "%",
                    BinOp::Eq => "==",
                    BinOp::Ne => "!=",
                    BinOp::Lt => "<",
                    BinOp::Le => "<=",
                    BinOp::Gt => ">",
                    BinOp::Ge => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                };
                format!("({l} {sym} {r})")
            }
            _ => "0".into(),
        }
    }

    fn emit_operand(&mut self, op: &Operand, _body: &MirBody) -> String {
        match op {
            Operand::Copy(p) | Operand::Move(p) => format!("_{}", p.local.index()),
            Operand::Constant(c) => match c {
                ConstValue::Int(n) => format!("{n}"),
                ConstValue::Bool(b) => format!("{}", i64::from(*b)),
                ConstValue::Float(f) => format!("{f}"),
                ConstValue::Str(s) => {
                    let idx = self.intern_string(s);
                    format!("rpy_str_{idx}")
                }
                ConstValue::Unit | ConstValue::ZeroSized => "0".into(),
            },
        }
    }

    fn emit_operand_place(&mut self, op: &OperandPlace) -> String {
        match op {
            OperandPlace::Place(p) => format!("_{}", p.local.index()),
            OperandPlace::ConstInt(n) => format!("{n}"),
            OperandPlace::ConstBool(b) => format!("{}", i64::from(*b)),
            OperandPlace::ConstStr(s) => {
                let idx = self.intern_string(s);
                format!("rpy_str_{idx}")
            }
        }
    }

    fn operand_place_ty(&self, op: &OperandPlace, body: &MirBody) -> TyKind {
        match op {
            OperandPlace::Place(p) => self.typed.types.kind(body.local_ty(p.local)).clone(),
            OperandPlace::ConstBool(_) => TyKind::Bool,
            OperandPlace::ConstInt(_) => TyKind::Int(rpython_types::IntWidth::I64),
            OperandPlace::ConstStr(_) => TyKind::Str,
        }
    }

    fn emit_string_pool(&mut self) {
        if self.strings.is_empty() {
            return;
        }
        for (i, s) in self.strings.iter().enumerate() {
            let escaped = escape_c(s);
            writeln!(
                self.out,
                "static const char rpy_str_lit_{i}[] = \"{escaped}\";"
            )
            .ok();
            writeln!(
                self.out,
                "static const rpy_str rpy_str_{i} = {{ rpy_str_lit_{i}, {} }};",
                s.len()
            )
            .ok();
        }
    }

    fn emit_main_wrapper(&mut self, mir: &MirCrate) {
        let has_main = mir.functions.values().any(|f| f.name.as_str() == "main");
        if has_main {
            self.out
                .push_str("int main(void) {\n  rpy_rt_init();\n  return (int)_rpy__main();\n}\n");
        }
    }

    fn c_type(&self, ty: rpython_ids::TypeId) -> &'static str {
        match self.typed.types.kind(ty) {
            TyKind::Bool => "int8_t",
            TyKind::Str => "rpy_str",
            TyKind::Unit | TyKind::Never => "void",
            TyKind::Int(_) => "int64_t",
            TyKind::Float(_) => "double",
            _ => "int64_t",
        }
    }
}

fn escape_c(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out
}
