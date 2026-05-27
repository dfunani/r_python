//! Lower typed AST to HIR (thin pass-through).

use rpython_ast::{
    Arena, BinaryOp as AstBinOp, ExprKind, ItemKind, Literal, Module, PatKind, StmtKind,
    UnaryOp as AstUnaryOp,
};
use rpython_hir::{
    BinaryOp, HirBody, HirConst, HirCrate, HirExpr, HirExprId, HirExprKind, HirOwner, HirOwnerKind,
    HirPat, HirPatId, HirPatKind, HirStmt, HirStmtId, HirStmtKind, LocalDecl, Operand, Place,
    Rvalue, UnaryOp,
};
use rpython_ids::{DefId, LocalId};
use rpython_resolve::resolve_path;
use rpython_resolve::DefKind;
use rpython_span::Span;
use rpython_typeck::TypedCrate;
use rpython_types::TyKind;
use rpython_types::{Mutability, Subst};
use smol_str::SmolStr;

struct HirBuilder {
    exprs: Vec<HirExpr>,
    stmts: Vec<HirStmt>,
    pats: Vec<HirPat>,
    locals: Vec<LocalDecl>,
    local_names: std::collections::HashMap<SmolStr, LocalId>,
    next_local: usize,
}

impl HirBuilder {
    fn new() -> Self {
        Self {
            exprs: Vec::new(),
            stmts: Vec::new(),
            pats: Vec::new(),
            locals: Vec::new(),
            local_names: std::collections::HashMap::new(),
            next_local: 0,
        }
    }

    fn alloc_local(
        &mut self,
        name: SmolStr,
        ty: rpython_types::TypeId,
        mutability: Mutability,
        span: Span,
    ) -> LocalId {
        let id = LocalId::from_usize(self.next_local);
        self.next_local += 1;
        self.locals.push(LocalDecl {
            ty,
            mutability,
            span,
        });
        self.local_names.insert(name, id);
        id
    }

    fn lookup_local(&self, name: &str) -> Option<LocalId> {
        self.local_names.get(name).copied()
    }

    fn alloc_expr(
        &mut self,
        kind: HirExprKind,
        ty: rpython_types::TypeId,
        span: Span,
    ) -> HirExprId {
        let id = HirExprId::from_usize(self.exprs.len());
        self.exprs.push(HirExpr { kind, ty, span });
        id
    }

    fn alloc_stmt(&mut self, kind: HirStmtKind, span: Span) -> HirStmtId {
        let id = HirStmtId::from_usize(self.stmts.len());
        self.stmts.push(HirStmt { kind, span });
        id
    }

    fn alloc_pat(&mut self, kind: HirPatKind, span: Span) -> HirPatId {
        let id = HirPatId::from_usize(self.pats.len());
        self.pats.push(HirPat { kind, span });
        id
    }
}

/// Build HIR for a type-checked module.
pub fn build_hir(typed: &TypedCrate, module: &Module, arena: &Arena) -> HirCrate {
    let mut hir_crate = HirCrate::default();
    let unit = typed.unit;
    let mut lowered = std::collections::HashSet::new();

    for &(item_id, def_id) in &typed.resolved.item_def_ids {
        let item = arena.item(item_id);
        if let ItemKind::Function {
            name,
            params,
            ret_ty,
            body,
            ..
        } = &item.kind
        {
            if lowered.insert(def_id) {
                build_one_function(
                    &mut hir_crate,
                    typed,
                    arena,
                    def_id,
                    name,
                    params,
                    *ret_ty,
                    body,
                    unit,
                );
            }
        }
    }

    for &item_id in &module.items {
        let item = arena.item(item_id);
        match &item.kind {
            ItemKind::Class { name, body, .. } => {
                if let Some(owner) = typed.resolved.def_map.lookup(typed.resolved.root, name) {
                    for &nested in body {
                        if let ItemKind::Function {
                            name: m,
                            params,
                            ret_ty,
                            body,
                            ..
                        } = &arena.item(nested).kind
                        {
                            if let Some(def) = typed.resolved.def_map.lookup(owner, m) {
                                if lowered.insert(def) {
                                    build_one_function(
                                        &mut hir_crate,
                                        typed,
                                        arena,
                                        def,
                                        m,
                                        params,
                                        *ret_ty,
                                        body,
                                        unit,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            ItemKind::Impl { self_ty, items, .. } => {
                let self_name = type_name_from_ty(*self_ty, arena);
                let impl_def = typed
                    .resolved
                    .def_map
                    .iter()
                    .find_map(|(def, kind)| match kind {
                        DefKind::Impl { self_ty_name, .. }
                            if self_ty_name.as_str() == self_name =>
                        {
                            Some(def)
                        }
                        _ => None,
                    });
                if let Some(owner) = impl_def {
                    for impl_item in items {
                        if let rpython_ast::ImplItem::Function {
                            name,
                            params,
                            ret_ty,
                            body,
                            ..
                        } = impl_item
                        {
                            if let Some(def) = typed.resolved.def_map.lookup(owner, name) {
                                if lowered.insert(def) {
                                    build_one_function(
                                        &mut hir_crate,
                                        typed,
                                        arena,
                                        def,
                                        name,
                                        params,
                                        *ret_ty,
                                        body,
                                        unit,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    hir_crate
}

#[allow(clippy::too_many_arguments)]
fn build_one_function(
    hir_crate: &mut HirCrate,
    typed: &TypedCrate,
    arena: &Arena,
    def_id: DefId,
    name: &SmolStr,
    params: &[rpython_ast::Param],
    ret_ty: Option<rpython_ids::TyId>,
    body: &[rpython_ids::StmtId],
    unit: rpython_types::TypeId,
) {
    let mut b = HirBuilder::new();
    let ret = typed.fn_ret.get(&def_id).copied().unwrap_or(unit);
    let param_tys = typed.fn_params.get(&def_id).cloned().unwrap_or_default();
    let mut param_locals = Vec::new();

    for (i, p) in params.iter().enumerate() {
        let ty = param_tys.get(i).copied().unwrap_or(unit);
        let local = b.alloc_local(p.name.clone(), ty, Mutability::Imm, p.span);
        param_locals.push(local);
    }

    let mut stmt_ids = Vec::new();
    for &stmt_id in body {
        stmt_ids.push(lower_stmt(&mut b, typed, arena, stmt_id));
    }

    let body = HirBody {
        def_id,
        name: name.clone(),
        params: param_locals,
        ret_ty: ret_ty.map(|_| ret).unwrap_or(ret),
        stmts: stmt_ids,
        exprs: b.exprs,
        stmts_data: b.stmts,
        pats: b.pats,
        locals: b.locals,
    };

    hir_crate.owners.insert(
        def_id,
        HirOwner {
            def_id,
            kind: HirOwnerKind::Function(body),
        },
    );
}

fn resolve_method_def(typed: &TypedCrate, recv_ty: rpython_types::TypeId, method: &str) -> DefId {
    if let Some(d) = typed.impl_table.find_method(recv_ty, method) {
        return d;
    }
    if let TyKind::Adt { def, .. } = typed.interner().kind(recv_ty) {
        if let Some(d) = typed.resolved.def_map.lookup(*def, method) {
            return d;
        }
    }
    DefId::from_usize(0)
}

fn type_name_from_ty(ty: rpython_ids::TyId, arena: &Arena) -> String {
    match &arena.ty(ty).kind {
        rpython_ast::TyKind::Path(p) => p
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_else(|| "_".into()),
        _ => "_".into(),
    }
}

fn lower_stmt(
    b: &mut HirBuilder,
    typed: &TypedCrate,
    arena: &Arena,
    stmt_id: rpython_ids::StmtId,
) -> HirStmtId {
    let stmt = arena.stmt(stmt_id);
    let unit = typed.unit;

    match &stmt.kind {
        StmtKind::Expr(expr) => {
            let e = lower_expr(b, typed, arena, *expr);
            b.alloc_stmt(HirStmtKind::Expr(e), stmt.span)
        }
        StmtKind::Assign { targets, value } => {
            if let Some(&pat_id) = targets.first() {
                let val_ty = typed.expr_types.get(value).copied().unwrap_or(unit);
                let place = lower_pat_assign(b, typed, arena, pat_id, val_ty, stmt.span);
                let rvalue = lower_expr_to_rvalue(b, typed, arena, *value);
                return b.alloc_stmt(HirStmtKind::Assign { place, rvalue }, stmt.span);
            }
            let val = lower_expr(b, typed, arena, *value);
            b.alloc_stmt(HirStmtKind::Expr(val), stmt.span)
        }
        StmtKind::AnnAssign { target, value, .. } => {
            if let Some(v) = value {
                let val_ty = typed.expr_types.get(v).copied().unwrap_or(unit);
                let place = lower_pat_assign(b, typed, arena, *target, val_ty, stmt.span);
                let rvalue = lower_expr_to_rvalue(b, typed, arena, *v);
                return b.alloc_stmt(HirStmtKind::Assign { place, rvalue }, stmt.span);
            }
            let pat_ty = typed.pat_types.get(target).copied().unwrap_or(unit);
            lower_pat_assign(b, typed, arena, *target, pat_ty, stmt.span);
            b.alloc_stmt(HirStmtKind::Expr(HirExprId::from_usize(0)), stmt.span)
        }
        StmtKind::Return(opt) => {
            let e = opt.map(|ex| lower_expr(b, typed, arena, ex));
            b.alloc_stmt(HirStmtKind::Return(e), stmt.span)
        }
        StmtKind::If {
            test,
            then_body,
            elifs,
            else_body,
        } => {
            let cond = lower_expr(b, typed, arena, *test);
            for &s in then_body {
                lower_stmt(b, typed, arena, s);
            }
            for arm in elifs {
                lower_expr(b, typed, arena, arm.test);
                for &s in &arm.body {
                    lower_stmt(b, typed, arena, s);
                }
            }
            if let Some(body) = else_body {
                for &s in body {
                    lower_stmt(b, typed, arena, s);
                }
            }
            b.alloc_stmt(HirStmtKind::Expr(cond), stmt.span)
        }
        StmtKind::While { test, body } => {
            let cond = lower_expr(b, typed, arena, *test);
            let mut body_stmts = Vec::new();
            for &s in body {
                body_stmts.push(lower_stmt(b, typed, arena, s));
            }
            b.alloc_stmt(
                HirStmtKind::While {
                    cond,
                    body: body_stmts,
                },
                stmt.span,
            )
        }
        StmtKind::For { pat, iter, body } => {
            lower_expr(b, typed, arena, *iter);
            let pat_ty = typed.pat_types.get(pat).copied().unwrap_or(unit);
            lower_pat_assign(b, typed, arena, *pat, pat_ty, stmt.span);
            for &s in body {
                lower_stmt(b, typed, arena, s);
            }
            b.alloc_stmt(HirStmtKind::Expr(HirExprId::from_usize(0)), stmt.span)
        }
        StmtKind::Match { scrutinee, arms } => {
            let scr = lower_expr(b, typed, arena, *scrutinee);
            let mut hir_arms = Vec::new();
            for arm in arms {
                let pat = lower_pat(b, typed, arena, arm.pat);
                for &s in &arm.body {
                    lower_stmt(b, typed, arena, s);
                }
                hir_arms.push((pat, scr));
            }
            let match_expr = b.alloc_expr(
                HirExprKind::Match {
                    scrutinee: scr,
                    arms: hir_arms,
                },
                typed.expr_types.get(scrutinee).copied().unwrap_or(unit),
                stmt.span,
            );
            b.alloc_stmt(HirStmtKind::Expr(match_expr), stmt.span)
        }
        _ => b.alloc_stmt(HirStmtKind::Expr(HirExprId::from_usize(0)), stmt.span),
    }
}

fn lower_pat_assign(
    b: &mut HirBuilder,
    _typed: &TypedCrate,
    arena: &Arena,
    pat_id: rpython_ids::PatId,
    ty: rpython_types::TypeId,
    span: Span,
) -> Place {
    let pat = arena.pat(pat_id);
    match &pat.kind {
        PatKind::Ident {
            name, mutability, ..
        } => {
            let mutability = match mutability {
                rpython_ast::PatMutability::Imm => Mutability::Imm,
                rpython_ast::PatMutability::Mut => Mutability::Mut,
            };
            if let Some(local) = b.lookup_local(name.as_str()) {
                return Place::local(local);
            }
            let local = b.alloc_local(name.clone(), ty, mutability, span);
            Place::local(local)
        }
        _ => Place::local(LocalId::from_usize(0)),
    }
}

fn lower_pat(
    b: &mut HirBuilder,
    typed: &TypedCrate,
    arena: &Arena,
    pat_id: rpython_ids::PatId,
) -> HirPatId {
    let pat = arena.pat(pat_id);
    let unit = typed.unit;
    match &pat.kind {
        PatKind::Wild => b.alloc_pat(HirPatKind::Wild, pat.span),
        PatKind::Literal(lit) => b.alloc_pat(HirPatKind::Literal(ast_literal(lit)), pat.span),
        PatKind::Ident { name, .. } => {
            let local = b
                .lookup_local(name.as_str())
                .unwrap_or_else(|| b.alloc_local(name.clone(), unit, Mutability::Imm, pat.span));
            b.alloc_pat(
                HirPatKind::Local {
                    name: name.clone(),
                    local,
                },
                pat.span,
            )
        }
        PatKind::Enum {
            path, variant: _, ..
        } => {
            let def = resolve_path(path, &typed.resolved).unwrap_or(DefId::from_usize(0));
            b.alloc_pat(
                HirPatKind::Enum {
                    def,
                    variant: 0,
                    subpats: vec![],
                },
                pat.span,
            )
        }
        _ => b.alloc_pat(HirPatKind::Wild, pat.span),
    }
}

fn lower_expr(
    b: &mut HirBuilder,
    typed: &TypedCrate,
    arena: &Arena,
    expr_id: rpython_ids::ExprId,
) -> HirExprId {
    let expr = arena.expr(expr_id);
    let unit = typed.unit;
    let ty = typed.expr_types.get(&expr_id).copied().unwrap_or(unit);

    let kind = match &expr.kind {
        ExprKind::Literal(lit) => HirExprKind::Literal(ast_literal(lit)),
        ExprKind::Path(path) => {
            if let Some(def) = resolve_path(path, &typed.resolved) {
                HirExprKind::Path {
                    def,
                    subst: Subst::default(),
                }
            } else if let Some(name) = path.is_simple_ident() {
                HirExprKind::Local(
                    b.lookup_local(name.as_str())
                        .unwrap_or(LocalId::from_usize(0)),
                )
            } else {
                HirExprKind::Literal(HirConst::Unit)
            }
        }
        ExprKind::Unary { op, operand } => HirExprKind::Unary {
            op: match op {
                AstUnaryOp::Not => UnaryOp::Not,
                AstUnaryOp::Neg => UnaryOp::Neg,
                _ => UnaryOp::Neg,
            },
            operand: lower_expr(b, typed, arena, *operand),
        },
        ExprKind::Binary { op, left, right } => HirExprKind::Binary {
            op: lower_binop(*op),
            left: lower_expr(b, typed, arena, *left),
            right: lower_expr(b, typed, arena, *right),
        },
        ExprKind::Call { func, args, .. } => {
            let func_expr = arena.expr(*func);
            let def = if let ExprKind::Path(p) = &func_expr.kind {
                resolve_path(p, &typed.resolved).unwrap_or(DefId::from_usize(0))
            } else {
                DefId::from_usize(0)
            };
            if matches!(
                typed.resolved.def_map.get(def),
                Some(DefKind::Struct { .. })
            ) {
                HirExprKind::Struct {
                    def,
                    fields: args
                        .iter()
                        .enumerate()
                        .map(|(i, &a)| (i as u32, lower_expr(b, typed, arena, a)))
                        .collect(),
                }
            } else {
                HirExprKind::Call {
                    def,
                    subst: Subst::default(),
                    args: args
                        .iter()
                        .map(|&a| lower_expr(b, typed, arena, a))
                        .collect(),
                }
            }
        }
        ExprKind::If {
            test,
            then,
            else_branch,
        } => HirExprKind::If {
            cond: lower_expr(b, typed, arena, *test),
            then: lower_expr(b, typed, arena, *then),
            else_branch: lower_expr(b, typed, arena, *else_branch),
        },
        ExprKind::Tuple(elems) => HirExprKind::Tuple(
            elems
                .iter()
                .map(|&e| lower_expr(b, typed, arena, e))
                .collect(),
        ),
        ExprKind::Struct { path, fields } => {
            let def = resolve_path(path, &typed.resolved).unwrap_or(DefId::from_usize(0));
            HirExprKind::Struct {
                def,
                fields: fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| (i as u32, lower_expr(b, typed, arena, f.expr)))
                    .collect(),
            }
        }
        ExprKind::Field { base, field } => {
            let base_ty = typed.expr_types.get(base).copied().unwrap_or(unit);
            HirExprKind::Field {
                base: lower_expr(b, typed, arena, *base),
                field_index: field_index_for_type(typed, base_ty, field.as_str()),
            }
        }
        ExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            let recv_ty = typed.expr_types.get(receiver).copied().unwrap_or(unit);
            let method_def = resolve_method_def(typed, recv_ty, method.as_str());
            let mut call_args = vec![lower_expr(b, typed, arena, *receiver)];
            for &a in args {
                call_args.push(lower_expr(b, typed, arena, a));
            }
            HirExprKind::Call {
                def: method_def,
                subst: Subst::default(),
                args: call_args,
            }
        }
        ExprKind::Block(stmts) => {
            for &s in stmts {
                lower_stmt(b, typed, arena, s);
            }
            HirExprKind::Literal(HirConst::Unit)
        }
        _ => HirExprKind::Literal(HirConst::Unit),
    };

    b.alloc_expr(kind, ty, expr.span)
}

fn ast_literal(lit: &Literal) -> HirConst {
    match lit {
        Literal::Int(n) => HirConst::Int(*n),
        Literal::Bool(b) => HirConst::Bool(*b),
        Literal::Float(f) => HirConst::Float(*f),
        Literal::String(s) => HirConst::Str(s.clone()),
        _ => HirConst::Unit,
    }
}

fn lower_binop(op: AstBinOp) -> BinaryOp {
    match op {
        AstBinOp::Add => BinaryOp::Add,
        AstBinOp::Sub => BinaryOp::Sub,
        AstBinOp::Mul => BinaryOp::Mul,
        AstBinOp::Div => BinaryOp::Div,
        AstBinOp::Mod | AstBinOp::FloorDiv => BinaryOp::Mod,
        AstBinOp::Eq => BinaryOp::Eq,
        AstBinOp::NotEq => BinaryOp::NotEq,
        AstBinOp::Lt => BinaryOp::Lt,
        AstBinOp::LtEq => BinaryOp::LtEq,
        AstBinOp::Gt => BinaryOp::Gt,
        AstBinOp::GtEq => BinaryOp::GtEq,
        AstBinOp::And => BinaryOp::And,
        AstBinOp::Or => BinaryOp::Or,
        _ => BinaryOp::Add,
    }
}

fn field_index_for_type(typed: &TypedCrate, ty: rpython_types::TypeId, field: &str) -> u32 {
    if let TyKind::Adt { def, .. } = typed.interner().kind(ty) {
        if let Some(DefKind::Struct { fields, .. }) = typed.resolved.def_map.get(*def) {
            return fields.iter().position(|f| f.as_str() == field).unwrap_or(0) as u32;
        }
    }
    0
}

fn lower_expr_to_rvalue(
    b: &mut HirBuilder,
    typed: &TypedCrate,
    arena: &Arena,
    expr_id: rpython_ids::ExprId,
) -> Rvalue {
    let expr = arena.expr(expr_id);
    match &expr.kind {
        ExprKind::Struct { path, fields } => {
            let def = resolve_path(path, &typed.resolved).unwrap_or(DefId::from_usize(0));
            let ops: Vec<Operand> = fields
                .iter()
                .map(|f| operand_from_expr(b, typed, arena, f.expr))
                .collect();
            Rvalue::Aggregate(rpython_hir::AggregateKind::Struct(def), ops)
        }
        ExprKind::Unary { op, operand } => Rvalue::UnaryOp {
            op: match op {
                AstUnaryOp::Not => UnaryOp::Not,
                AstUnaryOp::Neg => UnaryOp::Neg,
                _ => UnaryOp::Neg,
            },
            operand: operand_from_expr(b, typed, arena, *operand),
        },
        ExprKind::Binary { op, left, right } => Rvalue::BinaryOp {
            op: lower_binop(*op),
            left: operand_from_expr(b, typed, arena, *left),
            right: operand_from_expr(b, typed, arena, *right),
        },
        ExprKind::Literal(lit) => Rvalue::Use(Operand::Constant(hir_const_from_ast(lit))),
        ExprKind::Path(path) => {
            if let Some(name) = path.is_simple_ident() {
                if let Some(local) = b.lookup_local(name.as_str()) {
                    return Rvalue::Use(Operand::Copy(Place::local(local)));
                }
            }
            Rvalue::Use(Operand::Constant(HirConst::Unit))
        }
        _ => {
            let e = lower_expr(b, typed, arena, expr_id);
            hir_expr_to_rvalue(b, e)
        }
    }
}

fn hir_expr_to_rvalue(b: &HirBuilder, expr: HirExprId) -> Rvalue {
    match &b.exprs[expr.index()].kind {
        HirExprKind::Literal(c) => Rvalue::Use(Operand::Constant(c.clone())),
        HirExprKind::Local(l) => Rvalue::Use(Operand::Copy(Place::local(*l))),
        HirExprKind::Unary { op, operand } => Rvalue::UnaryOp {
            op: *op,
            operand: Operand::Copy(hir_operand_place(b, *operand)),
        },
        HirExprKind::Binary { op, left, right } => Rvalue::BinaryOp {
            op: *op,
            left: Operand::Copy(hir_operand_place(b, *left)),
            right: Operand::Copy(hir_operand_place(b, *right)),
        },
        HirExprKind::Struct { def, fields } => Rvalue::Aggregate(
            rpython_hir::AggregateKind::Struct(*def),
            fields
                .iter()
                .map(|(_, e)| Operand::Copy(hir_operand_place(b, *e)))
                .collect(),
        ),
        _ => Rvalue::Use(Operand::Constant(HirConst::Unit)),
    }
}

fn hir_operand_place(b: &HirBuilder, expr: HirExprId) -> Place {
    match &b.exprs[expr.index()].kind {
        HirExprKind::Local(l) => Place::local(*l),
        HirExprKind::Field { base, field_index } => {
            let base_place = hir_operand_place(b, *base);
            Place {
                local: base_place.local,
                projection: {
                    let mut p = base_place.projection;
                    p.push(rpython_hir::Projection::Field(*field_index));
                    p
                },
            }
        }
        _ => Place::local(LocalId::from_usize(0)),
    }
}

fn operand_from_expr(
    b: &mut HirBuilder,
    typed: &TypedCrate,
    arena: &Arena,
    expr_id: rpython_ids::ExprId,
) -> Operand {
    let expr = arena.expr(expr_id);
    match &expr.kind {
        ExprKind::Literal(lit) => Operand::Constant(hir_const_from_ast(lit)),
        ExprKind::Path(path) => {
            if let Some(name) = path.is_simple_ident() {
                if let Some(local) = b.lookup_local(name.as_str()) {
                    return Operand::Copy(Place::local(local));
                }
            }
            Operand::Constant(HirConst::Unit)
        }
        _ => {
            let e = lower_expr(b, typed, arena, expr_id);
            Operand::Copy(hir_operand_place(b, e))
        }
    }
}

fn hir_const_from_ast(lit: &Literal) -> HirConst {
    ast_literal(lit)
}
