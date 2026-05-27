use crate::{
    Arena, ElifArm, Expr, ExprId, ExprKind, ExternItem, FieldDef, FieldExpr, GenericParam, ImplItem,
    Item, ItemId, ItemKind, Kwarg, MatchArm, Param, Pat, PatId, PatKind, PatField,
    InterfaceItem, Stmt, StmtId, StmtKind, Ty, TyId, TyKind, Variant, VariantFields,
};
use crate::{Module, Path, PathSegment};

/// Visitor over AST nodes stored in an arena.
pub trait Visitor {
    fn visit_module(&mut self, _module: &Module, _arena: &Arena) {}
    fn visit_item(&mut self, _item: &Item, _arena: &Arena) {}
    fn visit_stmt(&mut self, _stmt: &Stmt, _arena: &Arena) {}
    fn visit_expr(&mut self, _expr: &Expr, _arena: &Arena) {}
    fn visit_pat(&mut self, _pat: &Pat, _arena: &Arena) {}
    fn visit_ty(&mut self, _ty: &Ty, _arena: &Arena) {}
}

/// Walk a module and all nested nodes.
pub fn walk_module<V: Visitor + ?Sized>(v: &mut V, module: &Module, arena: &Arena) {
    v.visit_module(module, arena);
    for &item in &module.items {
        walk_item(v, item, arena);
    }
}

/// Walk an item and nested bodies.
pub fn walk_item<V: Visitor + ?Sized>(v: &mut V, id: ItemId, arena: &Arena) {
    let item = arena.item(id);
    v.visit_item(item, arena);
    match &item.kind {
        ItemKind::Function { body, .. } => {
            for &stmt in body {
                walk_stmt(v, stmt, arena);
            }
        }
        ItemKind::Class { body, .. } => {
            for &nested in body {
                walk_item(v, nested, arena);
            }
        }
        ItemKind::Const { value, .. } => walk_expr(v, *value, arena),
        ItemKind::Module { items, .. } => {
            for &nested in items {
                walk_item(v, nested, arena);
            }
        }
        ItemKind::Impl { items, .. } => {
            for item in items {
                walk_impl_item(v, item, arena);
            }
        }
        ItemKind::Interface { items, .. } => {
            for item in items {
                walk_interface_item(v, item, arena);
            }
        }
        ItemKind::ExternBlock { items, .. } => {
            for item in items {
                walk_extern_item(v, item, arena);
            }
        }
        ItemKind::Struct { .. }
        | ItemKind::Enum { .. }
        | ItemKind::Import { .. } => {}
    }
}

fn walk_impl_item<V: Visitor + ?Sized>(v: &mut V, item: &ImplItem, arena: &Arena) {
    match item {
        ImplItem::Function { body, .. } => {
            for &stmt in body {
                walk_stmt(v, stmt, arena);
            }
        }
        ImplItem::Const { value, .. } => walk_expr(v, *value, arena),
        ImplItem::Type { .. } => {}
    }
}

fn walk_interface_item<V: Visitor + ?Sized>(v: &mut V, item: &InterfaceItem, arena: &Arena) {
    match item {
        InterfaceItem::Function { default_body, .. } => {
            if let Some(body) = default_body {
                for &stmt in body {
                    walk_stmt(v, stmt, arena);
                }
            }
        }
        InterfaceItem::Type { ty, .. } => {
            if let Some(ty) = ty {
                walk_ty(v, *ty, arena);
            }
        }
    }
}

fn walk_extern_item<V: Visitor + ?Sized>(_v: &mut V, _item: &ExternItem, _arena: &Arena) {}

/// Walk a statement and nested expressions/statements.
pub fn walk_stmt<V: Visitor + ?Sized>(v: &mut V, id: StmtId, arena: &Arena) {
    let stmt = arena.stmt(id);
    v.visit_stmt(stmt, arena);
    match &stmt.kind {
        StmtKind::Expr(expr) => walk_expr(v, *expr, arena),
        StmtKind::Assign { targets, value } => {
            for &pat in targets {
                walk_pat(v, pat, arena);
            }
            walk_expr(v, *value, arena);
        }
        StmtKind::AnnAssign { target, ty, value } => {
            walk_pat(v, *target, arena);
            walk_ty(v, *ty, arena);
            if let Some(val) = value {
                walk_expr(v, *val, arena);
            }
        }
        StmtKind::Return(expr) => {
            if let Some(expr) = expr {
                walk_expr(v, *expr, arena);
            }
        }
        StmtKind::Raise(expr) => walk_expr(v, *expr, arena),
        StmtKind::Assert { test, msg } => {
            walk_expr(v, *test, arena);
            if let Some(msg) = msg {
                walk_expr(v, *msg, arena);
            }
        }
        StmtKind::Pass | StmtKind::Break(_) | StmtKind::Continue(_) => {}
        StmtKind::While { test, body } => {
            walk_expr(v, *test, arena);
            walk_stmt_list(v, body, arena);
        }
        StmtKind::For { pat, iter, body } => {
            walk_pat(v, *pat, arena);
            walk_expr(v, *iter, arena);
            walk_stmt_list(v, body, arena);
        }
        StmtKind::If {
            test,
            then_body,
            elifs,
            else_body,
        } => {
            walk_expr(v, *test, arena);
            walk_stmt_list(v, then_body, arena);
            for elif in elifs {
                walk_elif(v, elif, arena);
            }
            if let Some(body) = else_body {
                walk_stmt_list(v, body, arena);
            }
        }
        StmtKind::Match { scrutinee, arms } => {
            walk_expr(v, *scrutinee, arena);
            for arm in arms {
                walk_match_arm(v, arm, arena);
            }
        }
    }
}

fn walk_stmt_list<V: Visitor + ?Sized>(v: &mut V, ids: &[StmtId], arena: &Arena) {
    for &id in ids {
        walk_stmt(v, id, arena);
    }
}

fn walk_elif<V: Visitor + ?Sized>(v: &mut V, elif: &ElifArm, arena: &Arena) {
    walk_expr(v, elif.test, arena);
    walk_stmt_list(v, &elif.body, arena);
}

fn walk_match_arm<V: Visitor + ?Sized>(v: &mut V, arm: &MatchArm, arena: &Arena) {
    walk_pat(v, arm.pat, arena);
    if let Some(guard) = arm.guard {
        walk_expr(v, guard, arena);
    }
    walk_stmt_list(v, &arm.body, arena);
}

/// Walk an expression and nested sub-expressions.
pub fn walk_expr<V: Visitor + ?Sized>(v: &mut V, id: ExprId, arena: &Arena) {
    let expr = arena.expr(id);
    v.visit_expr(expr, arena);
    match &expr.kind {
        ExprKind::Literal(_) | ExprKind::Path(_) => {}
        ExprKind::Call { func, args, kwargs } => {
            walk_expr(v, *func, arena);
            for &arg in args {
                walk_expr(v, arg, arena);
            }
            for kw in kwargs {
                walk_kwarg(v, kw, arena);
            }
        }
        ExprKind::MethodCall { receiver, args, .. } => {
            walk_expr(v, *receiver, arena);
            for &arg in args {
                walk_expr(v, arg, arena);
            }
        }
        ExprKind::Field { base, .. } => walk_expr(v, *base, arena),
        ExprKind::Index { base, index } => {
            walk_expr(v, *base, arena);
            walk_expr(v, *index, arena);
        }
        ExprKind::Unary { operand, .. } => walk_expr(v, *operand, arena),
        ExprKind::Binary { left, right, .. } => {
            walk_expr(v, *left, arena);
            walk_expr(v, *right, arena);
        }
        ExprKind::Tuple(elems) | ExprKind::List(elems) => {
            for &elem in elems {
                walk_expr(v, elem, arena);
            }
        }
        ExprKind::Struct { path, fields } => {
            walk_path(v, path, arena);
            for field in fields {
                walk_field_expr(v, field, arena);
            }
        }
        ExprKind::If { test, then, else_branch } => {
            walk_expr(v, *test, arena);
            walk_expr(v, *then, arena);
            walk_expr(v, *else_branch, arena);
        }
        ExprKind::Block(stmts) => walk_stmt_list(v, stmts, arena),
        ExprKind::Lambda { params, body } => {
            for param in params {
                if let Some(ty) = param.ty {
                    walk_ty(v, ty, arena);
                }
            }
            walk_expr(v, *body, arena);
        }
        ExprKind::Cast { expr, ty } => {
            walk_expr(v, *expr, arena);
            walk_ty(v, *ty, arena);
        }
        ExprKind::Ref { expr, .. } => walk_expr(v, *expr, arena),
        ExprKind::Deref(expr) => walk_expr(v, *expr, arena),
    }
}

fn walk_kwarg<V: Visitor + ?Sized>(v: &mut V, kw: &Kwarg, arena: &Arena) {
    walk_expr(v, kw.value, arena);
}

fn walk_field_expr<V: Visitor + ?Sized>(v: &mut V, field: &FieldExpr, arena: &Arena) {
    walk_expr(v, field.expr, arena);
}

/// Walk a pattern and nested sub-patterns.
pub fn walk_pat<V: Visitor + ?Sized>(v: &mut V, id: PatId, arena: &Arena) {
    let pat = arena.pat(id);
    v.visit_pat(pat, arena);
    match &pat.kind {
        PatKind::Wild | PatKind::Literal(_) => {}
        PatKind::Ident { subpat, .. } => {
            if let Some(sub) = subpat {
                walk_pat(v, *sub, arena);
            }
        }
        PatKind::Tuple(pats) | PatKind::Or(pats) => {
            for &p in pats {
                walk_pat(v, p, arena);
            }
        }
        PatKind::Struct { path, fields } => {
            walk_path(v, path, arena);
            for field in fields {
                walk_pat_field(v, field, arena);
            }
        }
        PatKind::Enum { path, subpats, .. } => {
            walk_path(v, path, arena);
            for &p in subpats {
                walk_pat(v, p, arena);
            }
        }
    }
}

fn walk_pat_field<V: Visitor + ?Sized>(v: &mut V, field: &PatField, arena: &Arena) {
    walk_pat(v, field.pat, arena);
}

/// Walk a type and nested type arguments.
pub fn walk_ty<V: Visitor + ?Sized>(v: &mut V, id: TyId, arena: &Arena) {
    let ty = arena.ty(id);
    v.visit_ty(ty, arena);
    match &ty.kind {
        TyKind::Path(path) => walk_path(v, path, arena),
        TyKind::Tuple(elems) => {
            for &elem in elems {
                walk_ty(v, elem, arena);
            }
        }
        TyKind::Array { elem, .. } | TyKind::Slice { elem } => walk_ty(v, *elem, arena),
        TyKind::Ref { inner, .. } => walk_ty(v, *inner, arena),
        TyKind::Fn { params, ret } => {
            for &param in params {
                walk_ty(v, param, arena);
            }
            if let Some(ret) = ret {
                walk_ty(v, *ret, arena);
            }
        }
        TyKind::GenericParam { .. } => {}
    }
}

fn walk_path<V: Visitor + ?Sized>(v: &mut V, path: &Path, arena: &Arena) {
    for seg in &path.segments {
        walk_path_segment(v, seg, arena);
    }
}

fn walk_path_segment<V: Visitor + ?Sized>(v: &mut V, seg: &PathSegment, arena: &Arena) {
    for &arg in &seg.args {
        walk_ty(v, arg, arena);
    }
}

/// Walk struct/enum definitions for nested types (fields, variants).
pub fn walk_field_defs<V: Visitor + ?Sized>(v: &mut V, fields: &[FieldDef], arena: &Arena) {
    for field in fields {
        walk_ty(v, field.ty, arena);
    }
}

pub fn walk_variants<V: Visitor + ?Sized>(v: &mut V, variants: &[Variant], arena: &Arena) {
    for variant in variants {
        walk_variant_fields(v, &variant.fields, arena);
    }
}

fn walk_variant_fields<V: Visitor + ?Sized>(v: &mut V, fields: &VariantFields, arena: &Arena) {
    match fields {
        VariantFields::Unit => {}
        VariantFields::Tuple(tys) => {
            for &ty in tys {
                walk_ty(v, ty, arena);
            }
        }
        VariantFields::Struct(defs) => walk_field_defs(v, defs, arena),
    }
}

pub fn walk_generic_params<V: Visitor + ?Sized>(v: &mut V, params: &[GenericParam], arena: &Arena) {
    for param in params {
        for &bound in &param.bounds {
            walk_ty(v, bound, arena);
        }
    }
}

pub fn walk_params<V: Visitor + ?Sized>(v: &mut V, params: &[Param], arena: &Arena) {
    for param in params {
        if let Some(ty) = param.ty {
            walk_ty(v, ty, arena);
        }
        if let Some(default) = param.default {
            walk_expr(v, default, arena);
        }
    }
}
