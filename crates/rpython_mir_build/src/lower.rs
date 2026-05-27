use rpython_hir::{
    HirBody, HirConst, HirExpr, HirExprKind, HirOwnerKind, HirStmt, HirStmtKind, Operand, Place,
    Rvalue, UnaryOp,
};
use rpython_ids::{BlockId, LocalId};
use rpython_mir::{
    BasicBlock, BinOp, ConstValue, FnOperand, LocalDecl, MirBody, Operand as MirOperand, Place as MirPlace,
    Rvalue as MirRvalue, StatementKind, Terminator, TerminatorKind, UnaryOp as MirUnaryOp,
};
use rpython_span::Span;
use rpython_types::Mutability;

use crate::builder::MirBuilder;

pub fn lower_function(body: &HirBody) -> MirBody {
    let span = body
        .locals
        .first()
        .map(|l| l.span)
        .unwrap_or_else(Span::dummy);
    let mut builder = MirBuilder::new(body.ret_ty, span);

    for &param in &body.params {
        let _ = param;
    }

    for &stmt_id in &body.stmts {
        lower_stmt(&mut builder, body, stmt_id);
    }

    if !matches!(
        builder.blocks.last().map(|b| &b.terminator.kind),
        Some(TerminatorKind::Return) | Some(TerminatorKind::Unreachable)
    ) {
        builder.terminate(
            TerminatorKind::Return,
            span,
        );
    }

    MirBody {
        name: body.name.clone(),
        def_id: body.def_id,
        arg_count: body.params.len(),
        return_ty: body.ret_ty,
        locals: builder.locals,
        blocks: builder.blocks,
        source_scopes: vec![],
    }
}

fn lower_stmt(builder: &mut MirBuilder, body: &HirBody, stmt_id: rpython_hir::HirStmtId) {
    let stmt = &body.stmts_data[stmt_id.index()];
    let span = stmt.span;
    match &stmt.kind {
        HirStmtKind::Assign { place, rvalue } => {
            let mir_place = lower_place(place);
            let mir_rv = lower_rvalue(builder, body, rvalue, span);
            builder.assign_rvalue(mir_place, mir_rv, span);
        }
        HirStmtKind::Expr(expr) => {
            let _ = lower_expr(builder, body, *expr, span);
        }
        HirStmtKind::Return(opt) => {
            if let Some(expr) = opt {
                let val = lower_expr(builder, body, *expr, span);
                let tmp = expr_local(body, val);
                builder.assign_use(MirPlace::return_place(), MirOperand::Copy(tmp), span);
            }
            builder.terminate(TerminatorKind::Return, span);
        }
        HirStmtKind::Drop(place) => {
            let _ = lower_place(place);
        }
    }
}

fn lower_expr(
    builder: &mut MirBuilder,
    body: &HirBody,
    id: rpython_hir::HirExprId,
    span: Span,
) -> MirPlace {
    let expr = &body.exprs[id.index()];
    match &expr.kind {
        HirExprKind::Literal(c) => {
            let local = builder.alloc_local(expr.ty, Mutability::Imm, span);
            builder.assign_const(MirPlace::local(local), hir_const(c), span);
            MirPlace::local(local)
        }
        HirExprKind::Local(local) => MirPlace::local(*local),
        HirExprKind::Path { def, .. } => {
            let local = builder.alloc_local(expr.ty, Mutability::Imm, span);
            let next = builder.new_block(span);
            builder.terminate(
                TerminatorKind::Call {
                    func: FnOperand::Def(*def),
                    args: vec![],
                    destination: Some(MirPlace::local(local)),
                    target: next,
                    unwind: None,
                },
                span,
            );
            builder.switch_to(next);
            MirPlace::local(local)
        }
        HirExprKind::Unary { op, operand } => {
            let op_place = lower_expr(builder, body, *operand, span);
            let local = builder.alloc_local(expr.ty, Mutability::Imm, span);
            builder.assign_rvalue(
                MirPlace::local(local),
                MirRvalue::UnaryOp {
                    op: match op {
                        UnaryOp::Not => MirUnaryOp::Not,
                        UnaryOp::Neg => MirUnaryOp::Neg,
                    },
                    operand: MirOperand::Copy(op_place),
                },
                span,
            );
            MirPlace::local(local)
        }
        HirExprKind::Binary { op, left, right } => {
            let l = lower_expr(builder, body, *left, span);
            let r = lower_expr(builder, body, *right, span);
            let local = builder.alloc_local(expr.ty, Mutability::Imm, span);
            builder.assign_rvalue(
                MirPlace::local(local),
                MirRvalue::BinaryOp {
                    op: lower_binop(*op),
                    left: MirOperand::Copy(l),
                    right: MirOperand::Copy(r),
                },
                span,
            );
            MirPlace::local(local)
        }
        HirExprKind::Call { def, args, .. } => {
            let mut mir_args = Vec::new();
            for &arg in args {
                let p = lower_expr(builder, body, arg, span);
                mir_args.push(rpython_mir::OperandPlace::Place(p));
            }
            let dest = builder.alloc_local(expr.ty, Mutability::Imm, span);
            let next = builder.new_block(span);
            builder.terminate(
                TerminatorKind::Call {
                    func: FnOperand::Def(*def),
                    args: mir_args,
                    destination: Some(MirPlace::local(dest)),
                    target: next,
                    unwind: None,
                },
                span,
            );
            builder.switch_to(next);
            MirPlace::local(dest)
        }
        HirExprKind::If {
            cond,
            then,
            else_branch,
        } => {
            let c = lower_expr(builder, body, *cond, span);
            let then_bb = builder.new_block(span);
            let else_bb = builder.new_block(span);
            let merge = builder.new_block(span);
            builder.terminate(
                TerminatorKind::SwitchInt {
                    discr: rpython_mir::OperandPlace::Place(c),
                    targets: vec![(1, then_bb)],
                    otherwise: else_bb,
                },
                span,
            );
            builder.switch_to(then_bb);
            let t = lower_expr(builder, body, *then, span);
            builder.assign_use(MirPlace::return_place(), MirOperand::Copy(t), span);
            builder.terminate(TerminatorKind::Goto { target: merge }, span);
            builder.switch_to(else_bb);
            let e = lower_expr(builder, body, *else_branch, span);
            builder.assign_use(MirPlace::return_place(), MirOperand::Copy(e), span);
            builder.terminate(TerminatorKind::Goto { target: merge }, span);
            builder.switch_to(merge);
            MirPlace::return_place()
        }
        _ => {
            let local = builder.alloc_local(expr.ty, Mutability::Imm, span);
            builder.assign_const(MirPlace::local(local), ConstValue::Unit, span);
            MirPlace::local(local)
        }
    }
}

fn lower_rvalue(
    builder: &mut MirBuilder,
    body: &HirBody,
    rv: &Rvalue,
    span: Span,
) -> MirRvalue {
    match rv {
        Rvalue::Use(op) => MirRvalue::Use(lower_operand(op)),
        Rvalue::UnaryOp { op, operand } => MirRvalue::UnaryOp {
            op: match op {
                UnaryOp::Not => MirUnaryOp::Not,
                UnaryOp::Neg => MirUnaryOp::Neg,
            },
            operand: lower_operand(operand),
        },
        Rvalue::BinaryOp { op, left, right } => MirRvalue::BinaryOp {
            op: lower_binop(*op),
            left: lower_operand(left),
            right: lower_operand(right),
        },
        Rvalue::Aggregate(_, _) | Rvalue::Ref { .. } | Rvalue::Len(_) | Rvalue::Discriminant(_) => {
            MirRvalue::Use(MirOperand::Constant(ConstValue::Unit))
        }
    }
}

fn lower_operand(op: &Operand) -> MirOperand {
    match op {
        Operand::Copy(p) | Operand::Move(p) => MirOperand::Copy(lower_place(p)),
        Operand::Constant(c) => MirOperand::Constant(hir_const(c)),
    }
}

fn lower_place(p: &Place) -> MirPlace {
    MirPlace {
        local: p.local,
        projection: Vec::new(),
    }
}

fn expr_local(_body: &HirBody, place: MirPlace) -> MirPlace {
    place
}

fn hir_const(c: &HirConst) -> ConstValue {
    match c {
        HirConst::Int(n) => ConstValue::Int(*n),
        HirConst::Bool(b) => ConstValue::Bool(*b),
        HirConst::Float(f) => ConstValue::Float(*f),
        HirConst::Str(s) => ConstValue::Str(s.to_string()),
        HirConst::Unit => ConstValue::Unit,
    }
}

fn lower_binop(op: rpython_hir::BinaryOp) -> BinOp {
    match op {
        rpython_hir::BinaryOp::Add => BinOp::Add,
        rpython_hir::BinaryOp::Sub => BinOp::Sub,
        rpython_hir::BinaryOp::Mul => BinOp::Mul,
        rpython_hir::BinaryOp::Div => BinOp::Div,
        rpython_hir::BinaryOp::Eq => BinOp::Eq,
        rpython_hir::BinaryOp::NotEq => BinOp::Ne,
        rpython_hir::BinaryOp::Lt => BinOp::Lt,
        rpython_hir::BinaryOp::LtEq => BinOp::Le,
        rpython_hir::BinaryOp::Gt => BinOp::Gt,
        rpython_hir::BinaryOp::GtEq => BinOp::Ge,
        rpython_hir::BinaryOp::And => BinOp::And,
        rpython_hir::BinaryOp::Or => BinOp::Or,
    }
}
