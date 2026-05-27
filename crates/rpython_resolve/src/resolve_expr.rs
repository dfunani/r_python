use crate::def_map::DefMap;
use crate::ribs::RibStack;
use crate::symbols::NameBinding;
use rpython_ast::{
    Arena, ExprId, ExprKind, ImplItem, ItemId, ItemKind, PatKind, StmtId, StmtKind,
};
use rpython_errors::{Diagnostic, ErrorCode, Handler};
use rpython_ids::DefId;
use rustc_hash::FxHashMap;

/// Second pass: resolve names in expressions and statements.
pub struct ExprResolver<'a> {
    pub def_map: &'a mut DefMap,
    pub ribs: &'a mut RibStack,
    pub handler: &'a mut Handler,
    pub symbol_map: &'a mut FxHashMap<ExprId, NameBinding>,
    pub current_fn: Option<DefId>,
    pub module_parent: DefId,
}

impl<'a> ExprResolver<'a> {
    pub fn resolve_module(&mut self, items: &[ItemId], arena: &Arena) {
        for &item in items {
            self.resolve_item(item, arena);
        }
    }

    fn resolve_item(&mut self, id: ItemId, arena: &Arena) {
        let item = arena.item(id);
        match &item.kind {
            ItemKind::Function {
                name,
                params,
                body,
                ..
            } => {
                if let Some(def) = self.def_map.lookup(self.module_parent, name) {
                    self.resolve_function(def, params, body, arena);
                }
            }
            ItemKind::Const { value, .. } => self.resolve_expr(*value, arena),
            ItemKind::Impl { items, .. } => {
                for impl_item in items {
                    if let ImplItem::Function {
                        name,
                        params,
                        body,
                        ..
                    } = impl_item
                    {
                        if let Some(def) = self.ribs.resolve(name) {
                            self.resolve_function(def, params, body, arena);
                        }
                    }
                }
            }
            ItemKind::Class { body, .. } | ItemKind::Module { items: body, .. } => {
                for &nested in body {
                    self.resolve_item(nested, arena);
                }
            }
            _ => {}
        }
    }

    fn resolve_function(
        &mut self,
        owner: DefId,
        params: &[rpython_ast::Param],
        body: &[StmtId],
        arena: &Arena,
    ) {
        self.current_fn = Some(owner);
        self.ribs
            .push(crate::scope::ScopeKind::Function, owner, Some(self.module_parent));
        for param in params {
            if let Some(def) = self.def_map.lookup(owner, &param.name) {
                let _ = self.ribs.define(param.name.clone(), def);
            }
        }
        for &stmt in body {
            self.resolve_stmt(stmt, arena);
        }
        self.ribs.pop();
        self.current_fn = None;
    }

    fn resolve_stmt(&mut self, id: StmtId, arena: &Arena) {
        let stmt = arena.stmt(id);
        match &stmt.kind {
            StmtKind::Expr(e) => self.resolve_expr(*e, arena),
            StmtKind::Assign { targets, value } => {
                for &pat in targets {
                    self.resolve_pat(pat, arena, true);
                }
                self.resolve_expr(*value, arena);
            }
            StmtKind::AnnAssign { target, value, .. } => {
                self.resolve_pat(*target, arena, true);
                if let Some(v) = value {
                    self.resolve_expr(*v, arena);
                }
            }
            StmtKind::Return(e) => {
                if let Some(e) = e {
                    self.resolve_expr(*e, arena);
                }
            }
            StmtKind::Raise(e) => self.resolve_expr(*e, arena),
            StmtKind::Assert { test, msg } => {
                self.resolve_expr(*test, arena);
                if let Some(m) = msg {
                    self.resolve_expr(*m, arena);
                }
            }
            StmtKind::While { test, body } => {
                self.resolve_expr(*test, arena);
                self.resolve_block(body, arena);
            }
            StmtKind::For { pat, iter, body } => {
                self.resolve_expr(*iter, arena);
                self.resolve_pat(*pat, arena, true);
                self.resolve_block(body, arena);
            }
            StmtKind::If {
                test,
                then_body,
                elifs,
                else_body,
            } => {
                self.resolve_expr(*test, arena);
                self.resolve_block(then_body, arena);
                for elif in elifs {
                    self.resolve_expr(elif.test, arena);
                    self.resolve_block(&elif.body, arena);
                }
                if let Some(else_b) = else_body {
                    self.resolve_block(else_b, arena);
                }
            }
            StmtKind::Match { scrutinee, arms } => {
                self.resolve_expr(*scrutinee, arena);
                for arm in arms {
                    self.resolve_pat(arm.pat, arena, false);
                    if let Some(g) = arm.guard {
                        self.resolve_expr(g, arena);
                    }
                    self.resolve_block(&arm.body, arena);
                }
            }
            StmtKind::Pass | StmtKind::Break(_) | StmtKind::Continue(_) => {}
        }
    }

    fn resolve_block(&mut self, stmts: &[StmtId], arena: &Arena) {
        self.ribs
            .push(crate::scope::ScopeKind::Block, self.current_fn.unwrap_or(self.module_parent), self.current_fn);
        for &stmt in stmts {
            self.resolve_stmt(stmt, arena);
        }
        self.ribs.pop();
    }

    fn resolve_pat(&mut self, id: rpython_ids::PatId, arena: &Arena, declare: bool) {
        let pat = arena.pat(id);
        match &pat.kind {
            PatKind::Ident { name, subpat, .. } => {
                if declare {
                    if self.ribs.resolve(name).is_some() {
                        // Assignment to an existing binding (parameter or local).
                    } else if let Some(owner) = self.current_fn {
                        let index = self.ribs.current().bindings.len() as u32;
                        let def = self.def_map.alloc(crate::def_map::DefKind::Local {
                            owner,
                            index,
                            name: name.clone(),
                        });
                        self.def_map.insert_name(owner, name.clone(), def);
                        if self.ribs.define(name.clone(), def).is_some() {
                            self.handler.emit(
                                Diagnostic::error(format!("duplicate binding `{name}`"))
                                    .with_code(ErrorCode::E0201)
                                    .with_label(pat.span, "duplicate binding", true),
                            );
                        }
                    }
                } else if self.ribs.resolve(name).is_none() {
                    self.unresolved(name, pat.span);
                }
                if let Some(sub) = subpat {
                    self.resolve_pat(*sub, arena, declare);
                }
            }
            PatKind::Tuple(pats) => {
                for &p in pats {
                    self.resolve_pat(p, arena, declare);
                }
            }
            PatKind::Struct { fields, .. } => {
                for f in fields {
                    self.resolve_pat(f.pat, arena, false);
                }
            }
            PatKind::Enum { subpats, .. } => {
                for &p in subpats {
                    self.resolve_pat(p, arena, false);
                }
            }
            PatKind::Or(pats) => {
                for &p in pats {
                    self.resolve_pat(p, arena, false);
                }
            }
            PatKind::Wild | PatKind::Literal(_) => {}
        }
    }

    fn resolve_expr(&mut self, id: ExprId, arena: &Arena) {
        let expr = arena.expr(id);
        match &expr.kind {
            ExprKind::Path(path) => {
                if let Some(name) = path.is_simple_ident() {
                    if let Some(def) = self.resolve_name(name) {
                        self.symbol_map.insert(id, NameBinding::new(def));
                    } else {
                        self.unresolved(name, expr.span);
                    }
                } else {
                    self.resolve_qualified_path(path, id, expr.span);
                }
            }
            ExprKind::Call { func, args, .. } => {
                self.resolve_expr(*func, arena);
                for &a in args {
                    self.resolve_expr(a, arena);
                }
            }
            ExprKind::MethodCall {
                receiver,
                method: _,
                args,
            } => {
                self.resolve_expr(*receiver, arena);
                for &a in args {
                    self.resolve_expr(a, arena);
                }
            }
            ExprKind::Field { base, .. } => self.resolve_expr(*base, arena),
            ExprKind::Index { base, index } => {
                self.resolve_expr(*base, arena);
                self.resolve_expr(*index, arena);
            }
            ExprKind::Unary { operand, .. } => self.resolve_expr(*operand, arena),
            ExprKind::Binary { left, right, .. } => {
                self.resolve_expr(*left, arena);
                self.resolve_expr(*right, arena);
            }
            ExprKind::Tuple(es) | ExprKind::List(es) => {
                for &e in es {
                    self.resolve_expr(e, arena);
                }
            }
            ExprKind::Struct { fields, path, .. } => {
                self.resolve_qualified_path(path, id, expr.span);
                for f in fields {
                    self.resolve_expr(f.expr, arena);
                }
            }
            ExprKind::If {
                test,
                then,
                else_branch,
            } => {
                self.resolve_expr(*test, arena);
                self.resolve_expr(*then, arena);
                self.resolve_expr(*else_branch, arena);
            }
            ExprKind::Block(stmts) => self.resolve_block(stmts, arena),
            ExprKind::Lambda { body, .. } => self.resolve_expr(*body, arena),
            ExprKind::Cast { expr, .. } => self.resolve_expr(*expr, arena),
            ExprKind::Ref { expr, .. } => self.resolve_expr(*expr, arena),
            ExprKind::Deref(e) => self.resolve_expr(*e, arena),
            ExprKind::Literal(_) => {}
        }
    }

    fn resolve_name(&self, name: &str) -> Option<DefId> {
        if let Some(def) = self.ribs.resolve(name) {
            return Some(def);
        }
        self.def_map.lookup(self.module_parent, name)
    }

    fn resolve_qualified_path(
        &mut self,
        path: &rpython_ast::Path,
        expr_id: ExprId,
        span: rpython_span::Span,
    ) {
        if path.segments.is_empty() {
            return;
        }
        let first = &path.segments[0].ident;
        let mut current = self.resolve_name(first);
        for seg in path.segments.iter().skip(1) {
            current = current.and_then(|parent| self.def_map.lookup(parent, &seg.ident));
        }
        if let Some(def) = current {
            self.symbol_map.insert(expr_id, NameBinding::new(def));
        } else {
            self.unresolved(first, span);
        }
    }

    fn unresolved(&mut self, name: &str, span: rpython_span::Span) {
        self.handler.emit(
            Diagnostic::error(format!("cannot resolve name `{name}`"))
                .with_code(ErrorCode::E0204)
                .with_label(span, "not found in this scope", true),
        );
    }
}
