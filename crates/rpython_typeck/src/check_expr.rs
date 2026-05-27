use crate::traits::MonoInstance;
use crate::TypeCtxt;
use rpython_ast::{
    Arena, BinaryOp, ExprId, ExprKind, FieldExpr, ImplItem, ItemId, ItemKind, Literal, Path,
    PatKind, StmtId, StmtKind, TyKind as AstTyKind, UnaryOp,
};
use rpython_errors::{Diagnostic, ErrorCode};
use rpython_ids::{DefId, TypeId};
use rpython_resolve::DefKind;
use rpython_types::{Subst, TyKind};

impl<'a> TypeCtxt<'a> {
    pub fn check_module_items(&mut self, items: &[ItemId]) {
        for &item in items {
            self.check_item(item);
        }
    }

    fn check_item(&mut self, id: ItemId) {
        let item = self.arena.item(id);
        match &item.kind {
            ItemKind::Function {
                name,
                params,
                ret_ty,
                body,
                ..
            } => {
                if let Some(def) = self.resolution.def_map.lookup(self.root, name) {
                    self.check_function(def, params, *ret_ty, body);
                }
            }
            ItemKind::Const { ty, value, .. } => {
                let expected = self.ast_ty_to_type(*ty);
                let got = self.check_expr(*value);
                let span = self.arena.expr(*value).span;
                let _ = self.infer.unify(&mut self.db, self.handler, expected, got, span);
            }
            ItemKind::Impl {
                self_ty,
                items,
                interface_ref,
                ..
            } => {
                let self_type = self.ast_ty_to_type(*self_ty);
                let mut methods = indexmap::IndexMap::new();
                let impl_def = self.find_impl_def_for_type(self_type);
                let interface_def = interface_ref
                    .as_ref()
                    .and_then(|p| self.resolution.def_map.lookup(self.root, p.segments.last()?.ident.as_str()));
                for impl_item in items {
                    if let ImplItem::Function { name, params, ret_ty, body, span, .. } =
                        impl_item
                    {
                        let method_def = impl_def
                            .and_then(|d| self.resolution.def_map.lookup(d, name))
                            .or_else(|| self.find_impl_method_def(self_type, name));
                        if let Some(def) = method_def {
                            methods.insert(name.clone(), def);
                            self.check_function(def, params, *ret_ty, body);
                        }
                        let _ = span;
                    }
                }
                if let Some(impl_id) = impl_def {
                    self.impl_table.register(crate::traits::ImplEntry {
                        impl_id: rpython_ids::ImplId(impl_id.0),
                        def_id: impl_id,
                        self_ty: self_type,
                        interface_ref: interface_def,
                        methods,
                    });
                }
            }
            ItemKind::Class { body, .. } | ItemKind::Module { items: body, .. } => {
                for &nested in body {
                    self.check_item(nested);
                }
            }
            _ => {}
        }
    }

    fn resolve_method(&self, recv_ty: TypeId, method: &str) -> Option<DefId> {
        if let Some(def) = self.impl_table.find_method(recv_ty, method) {
            return Some(def);
        }
        let recv_def = self.adt_def_id(recv_ty)?;
        for entry in self.impl_table.entries() {
            if self.adt_def_id(entry.self_ty) == Some(recv_def) {
                if let Some(&method_def) = entry.methods.get(method) {
                    return Some(method_def);
                }
            }
        }
        None
    }

    fn local_def_for_pat(&self, pat: rpython_ids::PatId) -> Option<DefId> {
        let pat = self.arena.pat(pat);
        if let PatKind::Ident { name, .. } = &pat.kind {
            if let Some(owner) = self.current_fn {
                return self.resolution.def_map.lookup(owner, name);
            }
        }
        None
    }

    fn adt_def_id(&self, ty: TypeId) -> Option<DefId> {
        if let TyKind::Adt { def, .. } = self.db.kind(ty) {
            Some(*def)
        } else {
            None
        }
    }

    fn method_on_adt(&self, ty: TypeId, method: &str) -> Option<DefId> {
        if let TyKind::Adt { def, .. } = self.db.kind(ty) {
            return self.resolution.def_map.lookup(*def, method);
        }
        None
    }

    fn find_impl_method_def(&self, self_ty: TypeId, method: &str) -> Option<DefId> {
        let ty_name = self.adt_def_id(self_ty).and_then(|d| self.resolution.def_map.name(d))?;
        for (def, kind) in self.resolution.def_map.iter() {
            if let DefKind::Impl { self_ty_name, .. } = kind {
                if self_ty_name.as_str() == ty_name.as_str() {
                    if let Some(m) = self.resolution.def_map.lookup(def, method) {
                        return Some(m);
                    }
                }
            }
        }
        None
    }

    pub(crate) fn find_impl_def_for_type(&self, ty: TypeId) -> Option<DefId> {
        let ty_name = match self.db.kind(ty) {
            TyKind::Adt { def, .. } => self.resolution.def_map.name(*def)?,
            _ => return None,
        };
        for (def, kind) in self.resolution.def_map.iter() {
            if let DefKind::Impl { self_ty_name, .. } = kind {
                if self_ty_name.as_str() == ty_name.as_str() {
                    return Some(def);
                }
            }
        }
        None
    }

    pub fn check_stmt(&mut self, id: StmtId) {
        let stmt = self.arena.stmt(id);
        match &stmt.kind {
            StmtKind::Expr(e) => {
                let ty = self.check_expr(*e);
                let _ = self.infer.unify(
                    &mut self.db,
                    self.handler,
                    ty,
                    self.wk.unit,
                    stmt.span,
                );
            }
            StmtKind::Assign { targets, value } => {
                let val_ty = self.check_expr(*value);
                for &pat in targets {
                    let _ = self.check_pat(pat, val_ty);
                    if let Some(def) = self.local_def_for_pat(pat) {
                        self.local_types.insert(def, val_ty);
                    }
                }
            }
            StmtKind::AnnAssign { target, ty, value } => {
                let expected = self.ast_ty_to_type(*ty);
                if let Some(v) = value {
                    let got = self.check_expr(*v);
                    let _ = self
                        .infer
                        .unify(&mut self.db, self.handler, expected, got, stmt.span);
                }
                let _ = self.check_pat(*target, expected);
            }
            StmtKind::Return(expr) => {
                self.return_checked = true;
                let ret = expr
                    .map(|e| self.check_expr(e))
                    .unwrap_or(self.wk.unit);
                if !self.infer.unify(
                    &mut self.db,
                    self.handler,
                    self.return_ty,
                    ret,
                    stmt.span,
                ) {
                    self.handler.emit(
                        Diagnostic::error("return type mismatch")
                            .with_code(ErrorCode::E0304)
                            .with_label(stmt.span, "return expression", true),
                    );
                }
            }
            StmtKind::Raise(e) => {
                let _ = self.check_expr(*e);
            }
            StmtKind::Assert { test, msg } => {
                let t = self.check_expr(*test);
                let _ = self
                    .infer
                    .unify(&mut self.db, self.handler, t, self.wk.bool, stmt.span);
                if let Some(m) = msg {
                    let _ = self.check_expr(*m);
                }
            }
            StmtKind::While { test, body } => {
                let t = self.check_expr(*test);
                let _ = self
                    .infer
                    .unify(&mut self.db, self.handler, t, self.wk.bool, stmt.span);
                for &s in body {
                    self.check_stmt(s);
                }
            }
            StmtKind::For { pat, iter, body } => {
                let iter_ty = self.check_expr(*iter);
                let elem = self.db.fresh_infer();
                let _ = self.infer.unify(
                    &mut self.db,
                    self.handler,
                    iter_ty,
                    self.wk.int,
                    stmt.span,
                );
                let _ = self.check_pat(*pat, elem);
                for &s in body {
                    self.check_stmt(s);
                }
            }
            StmtKind::If {
                test,
                then_body,
                elifs,
                else_body,
            } => {
                let t = self.check_expr(*test);
                let _ = self
                    .infer
                    .unify(&mut self.db, self.handler, t, self.wk.bool, stmt.span);
                for &s in then_body {
                    self.check_stmt(s);
                }
                for elif in elifs {
                    let t = self.check_expr(elif.test);
                    let _ = self.infer.unify(
                        &mut self.db,
                        self.handler,
                        t,
                        self.wk.bool,
                        elif.span,
                    );
                    for &s in &elif.body {
                        self.check_stmt(s);
                    }
                }
                if let Some(else_b) = else_body {
                    for &s in else_b {
                        self.check_stmt(s);
                    }
                }
            }
            StmtKind::Match { scrutinee, arms } => {
                let scrut_ty = self.check_expr(*scrutinee);
                let mut covered_variants = Vec::new();
                for arm in arms {
                    let arm_ty = self.check_pat(arm.pat, scrut_ty);
                    if let Some(v) = self.pat_variant_name(arm.pat) {
                        covered_variants.push(v);
                    }
                    if let Some(g) = arm.guard {
                        let gty = self.check_expr(g);
                        let _ = self.infer.unify(
                            &mut self.db,
                            self.handler,
                            gty,
                            self.wk.bool,
                            arm.span,
                        );
                    }
                    for &s in &arm.body {
                        self.check_stmt(s);
                    }
                    let _ = arm_ty;
                }
                self.check_match_exhaustive(scrut_ty, &covered_variants, stmt.span);
            }
            StmtKind::Pass | StmtKind::Break(_) | StmtKind::Continue(_) => {}
        }
    }

    pub fn check_expr(&mut self, id: ExprId) -> TypeId {
        if let Some(&ty) = self.expr_types.get(&id) {
            return ty;
        }
        let expr = self.arena.expr(id);
        let span = expr.span;
        let ty = match &expr.kind {
            ExprKind::Literal(lit) => self.literal_type(lit),
            ExprKind::Path(path) => self.check_path(id, path, span),
            ExprKind::Call { func, args, kwargs } => {
                let callee = self.check_expr(*func);
                if let TyKind::Adt { def, .. } = self.db.kind(callee).clone() {
                    if matches!(
                        self.resolution.def_map.get(def),
                        Some(DefKind::Struct { .. })
                    ) {
                        for &arg in args {
                            let _ = self.check_expr(arg);
                        }
                        for kw in kwargs {
                            let _ = self.check_expr(kw.value);
                        }
                        self.expr_types.insert(id, callee);
                        return callee;
                    }
                }
                let (param_tys, ret) = self.fn_ty_parts(callee, span);
                if args.len() != param_tys.len() {
                    self.handler.emit(
                        Diagnostic::error(format!(
                            "expected {} arguments, found {}",
                            param_tys.len(),
                            args.len()
                        ))
                        .with_code(ErrorCode::E0301)
                        .with_label(span, "argument count mismatch", true),
                    );
                }
                for (i, &arg) in args.iter().enumerate() {
                    let arg_ty = self.check_expr(arg);
                    if let Some(&expected) = param_tys.get(i) {
                        let _ = self.infer.unify(
                            &mut self.db,
                            self.handler,
                            expected,
                            arg_ty,
                            self.arena.expr(arg).span,
                        );
                    }
                }
                for kw in kwargs {
                    let _ = self.check_expr(kw.value);
                }
                if let Some(def) = self.expr_def(id) {
                    self.maybe_mono(def, &[]);
                }
                ret
            }
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
                let recv_ty = self.check_expr(*receiver);
                let resolved = self
                    .resolve_method(recv_ty, method)
                    .or_else(|| self.method_on_adt(recv_ty, method))
                    .or_else(|| self.resolution.def_map.lookup(self.root, method));
                if let Some(method_def) = resolved {
                    let callee = self.db.fn_def(method_def, Subst::empty());
                    let (param_tys, ret) = self.fn_ty_parts(callee, span);
                    let skip = if param_tys.is_empty() { 0 } else { 1 };
                    for (i, &arg) in args.iter().enumerate() {
                        let arg_ty = self.check_expr(arg);
                        if let Some(&expected) = param_tys.get(i + skip) {
                            let _ = self.infer.unify(
                                &mut self.db,
                                self.handler,
                                expected,
                                arg_ty,
                                self.arena.expr(arg).span,
                            );
                        }
                    }
                    self.maybe_mono(method_def, &[]);
                    ret
                } else {
                    self.handler.emit(
                        Diagnostic::error(format!("no method `{method}` found"))
                            .with_code(ErrorCode::E0306)
                            .with_label(span, "method not found", true),
                    );
                    self.db.error()
                }
            }
            ExprKind::Field { base, field } => {
                let base_ty = self.check_expr(*base);
                self.struct_field_type(base_ty, field)
            }
            ExprKind::Index { base, index } => {
                let base_ty = self.check_expr(*base);
                let idx_ty = self.check_expr(*index);
                let _ = self
                    .infer
                    .unify(&mut self.db, self.handler, idx_ty, self.wk.int, span);
                base_ty
            }
            ExprKind::Unary { op, operand } => {
                let ot = self.check_expr(*operand);
                match op {
                    UnaryOp::Not => {
                        let _ = self.infer.unify(
                            &mut self.db,
                            self.handler,
                            ot,
                            self.wk.bool,
                            span,
                        );
                        self.wk.bool
                    }
                    UnaryOp::Neg | UnaryOp::Pos => {
                        let _ = self.infer.unify(
                            &mut self.db,
                            self.handler,
                            ot,
                            self.wk.int,
                            span,
                        );
                        self.wk.int
                    }
                    UnaryOp::BitNot => ot,
                }
            }
            ExprKind::Binary { op, left, right } => {
                let lt = self.check_expr(*left);
                let rt = self.check_expr(*right);
                match op {
                    BinaryOp::And | BinaryOp::Or => {
                        let _ = self.infer.unify(
                            &mut self.db,
                            self.handler,
                            lt,
                            self.wk.bool,
                            span,
                        );
                        let _ = self.infer.unify(
                            &mut self.db,
                            self.handler,
                            rt,
                            self.wk.bool,
                            span,
                        );
                        self.wk.bool
                    }
                    BinaryOp::Eq
                    | BinaryOp::NotEq
                    | BinaryOp::Lt
                    | BinaryOp::LtEq
                    | BinaryOp::Gt
                    | BinaryOp::GtEq
                    | BinaryOp::Is => {
                        let _ = self.infer.unify(&mut self.db, self.handler, lt, rt, span);
                        self.wk.bool
                    }
                    BinaryOp::Add if self.is_str(lt) || self.is_str(rt) => {
                        let _ = self.infer.unify(
                            &mut self.db,
                            self.handler,
                            lt,
                            self.wk.str,
                            span,
                        );
                        let _ = self.infer.unify(
                            &mut self.db,
                            self.handler,
                            rt,
                            self.wk.str,
                            span,
                        );
                        self.wk.str
                    }
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod
                    | BinaryOp::FloorDiv
                    | BinaryOp::Pow => {
                        let _ = self.infer.unify(
                            &mut self.db,
                            self.handler,
                            lt,
                            self.wk.int,
                            span,
                        );
                        let _ = self.infer.unify(
                            &mut self.db,
                            self.handler,
                            rt,
                            self.wk.int,
                            span,
                        );
                        self.wk.int
                    }
                    BinaryOp::In => self.wk.bool,
                    _ => self.wk.int,
                }
            }
            ExprKind::Tuple(elems) => {
                let types: Vec<_> = elems.iter().map(|&e| self.check_expr(e)).collect();
                self.db.tuple(types)
            }
            ExprKind::List(elems) => {
                for &e in elems {
                    let _ = self.check_expr(e);
                }
                self.db.fresh_infer()
            }
            ExprKind::Struct { path, fields } => self.check_struct_expr(path, fields, span),
            ExprKind::If {
                test,
                then,
                else_branch,
            } => {
                let t = self.check_expr(*test);
                let _ = self
                    .infer
                    .unify(&mut self.db, self.handler, t, self.wk.bool, span);
                let ty1 = self.check_expr(*then);
                let ty2 = self.check_expr(*else_branch);
                let merged = self.db.fresh_infer();
                let _ = self.infer.unify(&mut self.db, self.handler, merged, ty1, span);
                let _ = self.infer.unify(&mut self.db, self.handler, merged, ty2, span);
                self.infer.resolve(&mut self.db, merged)
            }
            ExprKind::Block(stmts) => {
                for &s in stmts {
                    self.check_stmt(s);
                }
                self.wk.unit
            }
            ExprKind::Lambda { params, body } => {
                for p in params {
                    let _ = p;
                }
                self.check_expr(*body)
            }
            ExprKind::Cast { expr, ty } => {
                let _ = self.check_expr(*expr);
                self.ast_ty_to_type(*ty)
            }
            ExprKind::Ref { expr, .. } => self.check_expr(*expr),
            ExprKind::Deref(e) => self.check_expr(*e),
        };
        let resolved = self.infer.resolve(&mut self.db, ty);
        self.expr_types.insert(id, resolved);
        resolved
    }

    fn check_path(&mut self, id: ExprId, path: &Path, span: rpython_span::Span) -> TypeId {
        if let Some(binding) = self.resolution.expr_bindings.get(&id) {
            let def = binding.def;
            if let Some(ty) = self.wk.type_for_builtin_def(&self.db, def) {
                return ty;
            }
            if let Some(&sig) = self.item_sigs.get(&def) {
                return sig;
            }
            return self.def_to_type(def);
        }
        if let Some(name) = path.is_simple_ident() {
            if let Some(def) = self.resolution.def_map.lookup(self.root, name) {
                return self.def_to_type(def);
            }
        }
        self.handler.emit(
            Diagnostic::error("cannot resolve path type")
                .with_code(ErrorCode::E0302)
                .with_label(span, "unknown path", true),
        );
        self.db.error()
    }

    fn check_struct_expr(
        &mut self,
        path: &Path,
        fields: &[FieldExpr],
        span: rpython_span::Span,
    ) -> TypeId {
        let struct_ty = self.resolve_path_type(path, span);
        for f in fields {
            let expected = self.struct_field_type(struct_ty, &f.name);
            let got = self.check_expr(f.expr);
            let _ = self.infer.unify(
                &mut self.db,
                self.handler,
                expected,
                got,
                f.span,
            );
        }
        struct_ty
    }

    pub fn literal_type(&mut self, lit: &Literal) -> TypeId {
        match lit {
            Literal::Int(_) => self.wk.int,
            Literal::Float(_) => self.wk.float,
            Literal::String(_) => self.wk.str,
            Literal::Bytes(_) => self.db.intern(TyKind::Bytes),
            Literal::Bool(_) => self.wk.bool,
            Literal::Char(_) => self.db.intern(TyKind::Char),
            Literal::None => self.wk.unit,
        }
    }

    fn is_str(&self, ty: TypeId) -> bool {
        matches!(self.db.kind(ty), TyKind::Str)
    }

    fn expr_def(&self, id: ExprId) -> Option<DefId> {
        self.resolution
            .expr_bindings
            .get(&id)
            .map(|b| b.def)
    }

    fn fn_ty_parts(&mut self, callee: TypeId, _span: rpython_span::Span) -> (Vec<TypeId>, TypeId) {
        if let TyKind::FnDef { def, .. } = self.db.kind(callee) {
            return self.function_sig_parts(*def);
        }
        if callee == self.db.fn_def(self.wk.print, Subst::empty())
            || self.item_sigs.get(&self.wk.print).is_some()
        {
            return (vec![self.db.fresh_infer()], self.wk.unit);
        }
        (vec![], self.wk.unit)
    }

    fn def_to_type(&mut self, def: DefId) -> TypeId {
        if let Some(ty) = self.wk.type_for_builtin_def(&self.db, def) {
            return ty;
        }
        if let Some(&sig) = self.item_sigs.get(&def) {
            return sig;
        }
        match self.resolution.def_map.get(def).cloned() {
            Some(rpython_resolve::DefKind::Function { .. }) => {
                self.db.fn_def(def, Subst::empty())
            }
            Some(rpython_resolve::DefKind::Struct { .. })
            | Some(rpython_resolve::DefKind::Enum { .. }) => self.db.adt(def, Subst::empty()),
            Some(rpython_resolve::DefKind::BuiltinFn { .. }) => {
                self.db.fn_def(def, Subst::empty())
            }
            Some(rpython_resolve::DefKind::Local { .. })
            | Some(rpython_resolve::DefKind::Param { .. }) => self
                .local_types
                .get(&def)
                .copied()
                .unwrap_or(self.wk.int),
            _ => self.db.error(),
        }
    }

    pub fn resolve_path_type(&mut self, path: &Path, span: rpython_span::Span) -> TypeId {
        if let Some(name) = path.is_simple_ident() {
            if let Some(def) = self.resolution.def_map.lookup(self.root, name) {
                return self.def_to_type(def);
            }
        }
        self.handler.emit(
            Diagnostic::error("unknown type in path")
                .with_code(ErrorCode::E0302)
                .with_label(span, "unknown type", true),
        );
        self.db.error()
    }

    pub fn ast_ty_to_type(&mut self, id: rpython_ids::TyId) -> TypeId {
        let ty = self.arena.ty(id);
        match &ty.kind {
            AstTyKind::Path(p) => {
                if let Some(name) = p.is_simple_ident() {
                    if let Some(def) = self.resolution.def_map.lookup(self.root, name) {
                        return self.def_to_type(def);
                    }
                    match name.as_str() {
                        "int" => return self.wk.int,
                        "bool" => return self.wk.bool,
                        "str" => return self.wk.str,
                        "void" | "unit" => return self.wk.unit,
                        "float" => return self.wk.float,
                        _ => {}
                    }
                }
                self.resolve_path_type(p, ty.span)
            }
            AstTyKind::Tuple(elems) => {
                let types: Vec<_> = elems.iter().map(|&t| self.ast_ty_to_type(t)).collect();
                self.db.tuple(types)
            }
            AstTyKind::Ref { inner, .. } => self.ast_ty_to_type(*inner),
            AstTyKind::Fn { params, ret } => {
                let ps: Vec<_> = params.iter().map(|&t| self.ast_ty_to_type(t)).collect();
                let r = ret
                    .map(|t| self.ast_ty_to_type(t))
                    .unwrap_or(self.wk.unit);
                self.db.intern(TyKind::FnPtr {
                    sig: rpython_types::FnSig { params: ps, ret: r },
                })
            }
            AstTyKind::Array { elem, .. } => self.ast_ty_to_type(*elem),
            AstTyKind::Slice { elem } => self.ast_ty_to_type(*elem),
            AstTyKind::GenericParam { name } => {
                if let Some(def) = self.resolution.def_map.lookup(self.root, name) {
                    if let Some(DefKind::TypeAlias { .. }) = self.resolution.def_map.get(def) {
                        return self.db.generic_param(0);
                    }
                }
                self.db.generic_param(0)
            }
        }
    }

    fn check_match_exhaustive(
        &mut self,
        scrut_ty: TypeId,
        covered: &[smol_str::SmolStr],
        span: rpython_span::Span,
    ) {
        if let TyKind::Adt { def, .. } = self.db.kind(scrut_ty).clone() {
            if let Some(DefKind::Enum { variants, .. }) = self.resolution.def_map.get(def) {
                let missing: Vec<_> = variants
                    .iter()
                    .filter(|v| !covered.iter().any(|c| c == *v))
                    .map(|v| v.to_string())
                    .collect();
                if !missing.is_empty() && !covered.is_empty() {
                    self.handler.emit(
                        Diagnostic::error(format!(
                            "non-exhaustive match: missing variant(s) {}",
                            missing.join(", ")
                        ))
                        .with_code(ErrorCode::E0303)
                        .with_label(span, "match is not exhaustive", true),
                    );
                }
            }
        }
    }

    fn pat_variant_name(&self, pat: rpython_ids::PatId) -> Option<smol_str::SmolStr> {
        match &self.arena.pat(pat).kind {
            rpython_ast::PatKind::Enum { variant, .. } => Some(variant.clone()),
            _ => None,
        }
    }

    fn maybe_mono(&mut self, def: DefId, args: &[TypeId]) {
        let inst = MonoInstance {
            def_id: def,
            subst: Subst::from_args(args.to_vec()),
        };
        if !self.mono_instances.contains(&inst) {
            self.mono_instances.push(inst);
        }
    }
}
