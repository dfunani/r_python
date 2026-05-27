use crate::TypeCtxt;
use rpython_ast::{Arena, Param, StmtId};
use rpython_ids::{DefId, TypeId};
use rpython_resolve::DefKind;
use rpython_types::{Subst, TyKind};

impl<'a> TypeCtxt<'a> {
    pub fn check_function(
        &mut self,
        def: DefId,
        params: &[Param],
        ret_ty: Option<rpython_ids::TyId>,
        body: &[StmtId],
    ) {
        self.current_fn = Some(def);
        self.return_ty = ret_ty
            .map(|t| self.ast_ty_to_type(t))
            .unwrap_or(self.wk.unit);

        let mut param_tys = Vec::new();
        for param in params {
            let ty = param
                .ty
                .map(|t| self.ast_ty_to_type(t))
                .unwrap_or_else(|| self.db.fresh_infer());
            param_tys.push(ty);
            if let Some(param_def) = self.resolution.def_map.lookup(def, &param.name) {
                self.local_types.insert(param_def, ty);
            }
        }

        let fn_ty = self.db.fn_def(def, Subst::empty());
        let sig_ret = if let TyKind::FnDef { .. } = self.db.kind(fn_ty) {
            self.return_ty
        } else {
            self.return_ty
        };
        self.item_sigs.insert(def, fn_ty);

        for &stmt in body {
            self.check_stmt(stmt);
        }

        if !self.return_checked && !self.db.is_unit(self.return_ty) && !self.db.is_never(sig_ret) {
            let span = body
                .last()
                .map(|s| self.arena.stmt(*s).span)
                .unwrap_or_else(rpython_span::Span::dummy);
            let _ = self.infer.unify(
                &mut self.db,
                self.handler,
                self.return_ty,
                self.wk.unit,
                span,
            );
        }

        self.current_fn = None;
        self.return_checked = false;
    }

    pub fn collect_item_sigs(&mut self, arena: &Arena) {
        for (def, kind) in self.resolution.def_map.iter() {
            match kind {
                DefKind::Function { .. } => {
                    let sig = self.fn_sig_for_def(def, arena);
                    self.item_sigs.insert(def, sig);
                }
                DefKind::Struct { .. } => {
                    let ty = self.db.adt(def, Subst::empty());
                    self.item_sigs.insert(def, ty);
                }
                DefKind::Enum { .. } => {
                    let ty = self.db.adt(def, Subst::empty());
                    self.item_sigs.insert(def, ty);
                }
                DefKind::Const { .. } => {}
                _ => {}
            }
        }
    }

    fn fn_sig_for_def(&mut self, def: DefId, _arena: &Arena) -> TypeId {
        let (params, ret) = self.function_sig_parts(def);
        self.db.intern(TyKind::FnPtr {
            sig: rpython_types::FnSig { params, ret },
        })
    }

    pub fn function_sig_parts(&mut self, def: DefId) -> (Vec<TypeId>, TypeId) {
        if def == self.wk.print {
            return (vec![self.db.fresh_infer()], self.wk.unit);
        }
        for &(item_id, item_def) in &self.resolution.item_def_ids {
            if item_def != def {
                continue;
            }
            let item = self.arena.item(item_id);
            if let rpython_ast::ItemKind::Function { params, ret_ty, .. } = &item.kind {
                let param_tys: Vec<_> = params
                    .iter()
                    .map(|p| p.ty.map(|t| self.ast_ty_to_type(t)).unwrap_or(self.wk.int))
                    .collect();
                let ret = ret_ty
                    .map(|t| self.ast_ty_to_type(t))
                    .unwrap_or(self.wk.unit);
                return (param_tys, ret);
            }
        }
        for &(item_id, _) in &self.resolution.item_def_ids {
            let item = self.arena.item(item_id);
            if let rpython_ast::ItemKind::Impl { self_ty, items, .. } = &item.kind {
                let self_type = self.ast_ty_to_type(*self_ty);
                if let Some(impl_block) = self.find_impl_def_for_type(self_type) {
                    for impl_item in items {
                        if let rpython_ast::ImplItem::Function {
                            name,
                            params,
                            ret_ty,
                            ..
                        } = impl_item
                        {
                            if self.resolution.def_map.lookup(impl_block, name) == Some(def) {
                                let param_tys: Vec<_> = params
                                    .iter()
                                    .map(|p| {
                                        p.ty.map(|t| self.ast_ty_to_type(t)).unwrap_or(self.wk.int)
                                    })
                                    .collect();
                                let ret = ret_ty
                                    .map(|t| self.ast_ty_to_type(t))
                                    .unwrap_or(self.wk.unit);
                                return (param_tys, ret);
                            }
                        }
                    }
                }
            }
        }
        (vec![], self.wk.unit)
    }
}
