//! Lower typed AST to HIR (thin pass-through).

use rpython_ast::{
    Arena, BinaryOp as AstBinOp, ExprKind, ItemKind, Literal, Module, PatKind, StmtKind,
    UnaryOp as AstUnaryOp,
};
use rpython_hir::{
    AggregateKind, BinaryOp, HirBody, HirConst, HirCrate, HirExpr, HirExprId, HirExprKind,
    HirOwner, HirOwnerKind, HirPat, HirPatId, HirPatKind, HirStmt, HirStmtId, HirStmtKind,
    LocalDecl, Operand, Place, Rvalue, UnaryOp,
};
use rpython_ids::{DefId, LocalId};
use rpython_resolve::resolve_path;
use rpython_typeck::TypedCrate;
use rpython_types::{Mutability, Subst};
use rpython_span::Span;
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

    fn alloc_expr(&mut self, kind: HirExprKind, ty: rpython_types::TypeId, span: Span) -> HirExprId {
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
            let mut b = HirBuilder::new();
            let ret = typed.fn_ret.get(&def_id).copied().unwrap_or(unit);
            let mut param_locals = Vec::new();
            let param_tys = typed.fn_params.get(&def_id).cloned().unwrap_or_default();

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
    }

    hir_crate
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
                let val_ty = typed.expr_types.get(&value).copied().unwrap_or(unit);
                let place = lower_pat_assign(b, typed, arena, pat_id, val_ty, stmt.span);
                let val = lower_expr(b, typed, arena, *value);
                let src = match &b.exprs[val.index()].kind {
                    HirExprKind::Local(l) => Place::local(*l),
                    _ => Place::local(b.alloc_local(
                        SmolStr::new("_tmp"),
                        val_ty,
                        rpython_types::Mutability::Imm,
                        stmt.span,
                    )),
                };
                return b.alloc_stmt(
                    HirStmtKind::Assign {
                        place,
                        rvalue: Rvalue::Use(Operand::Copy(src)),
                    },
                    stmt.span,
                );
            }
            let val = lower_expr(b, typed, arena, *value);
            b.alloc_stmt(HirStmtKind::Expr(val), stmt.span)
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
            for &s in body {
                lower_stmt(b, typed, arena, s);
            }
            b.alloc_stmt(HirStmtKind::Expr(cond), stmt.span)
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
    typed: &TypedCrate,
    arena: &Arena,
    pat_id: rpython_ids::PatId,
    ty: rpython_types::TypeId,
    span: Span,
) -> Place {
    let pat = arena.pat(pat_id);
    match &pat.kind {
        PatKind::Ident { name, mutability, .. } => {
            let mutability = match mutability {
                rpython_ast::PatMutability::Imm => Mutability::Imm,
                rpython_ast::PatMutability::Mut => Mutability::Mut,
            };
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
        PatKind::Literal(lit) => {
            b.alloc_pat(HirPatKind::Literal(ast_literal(lit)), pat.span)
        }
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
        PatKind::Enum { path, variant, .. } => {
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
                HirExprKind::Local(b.lookup_local(name.as_str()).unwrap_or(LocalId::from_usize(0)))
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
            HirExprKind::Call {
                def,
                subst: Subst::default(),
                args: args
                    .iter()
                    .map(|&a| lower_expr(b, typed, arena, a))
                    .collect(),
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
        ExprKind::Field { base, field: _ } => HirExprKind::Field {
            base: lower_expr(b, typed, arena, *base),
            field_index: 0,
        },
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
