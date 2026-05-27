use crate::unify::InferCtxt;
use crate::TypeCtxt;
use rpython_ast::{Arena, PatId, PatKind};
use rpython_ids::TypeId;
use rpython_resolve::DefKind;
use rpython_types::{Subst, TyKind};

impl<'a> TypeCtxt<'a> {
    pub fn check_pat(&mut self, pat: PatId, expected: TypeId) -> TypeId {
        let pat_data = self.arena.pat(pat).clone();
        let span = pat_data.span;
        let ty = match pat_data.kind {
            PatKind::Wild => {
                let var = self.db.fresh_infer();
                let _ = self.infer.unify(
                    &mut self.db,
                    self.handler,
                    expected,
                    var,
                    span,
                );
                var
            }
            PatKind::Ident { name: _, mutability: _, subpat } => {
                let var = self.infer.resolve(&mut self.db, expected);
                if let Some(sub) = subpat {
                    let sub_ty = self.check_pat(sub, var);
                    self.pat_types.insert(sub, sub_ty);
                }
                var
            }
            PatKind::Literal(lit) => self.literal_type(&lit),
            PatKind::Tuple(pats) => {
                let mut elems = Vec::new();
                if let TyKind::Tuple(expected_elems) = self.db.kind(expected).clone() {
                    for (i, &p) in pats.iter().enumerate() {
                        let exp = expected_elems.get(i).copied().unwrap_or_else(|| self.db.error());
                        let t = self.check_pat(p, exp);
                        elems.push(t);
                    }
                } else {
                    for &p in &pats {
                        let infer_ty = self.db.fresh_infer();
                        elems.push(self.check_pat(p, infer_ty));
                    }
                }
                self.db.tuple(elems)
            }
            PatKind::Struct { path, fields } => {
                let struct_ty = self.resolve_path_type(&path, span);
                for field in fields {
                    let field_ty = self.struct_field_type(struct_ty, &field.name);
                    let t = self.check_pat(field.pat, field_ty);
                    self.pat_types.insert(field.pat, t);
                }
                struct_ty
            }
            PatKind::Enum { path, variant, subpats } => {
                let enum_ty = self.resolve_path_type(&path, span);
                let variant_ty = self.enum_variant_types(enum_ty, &variant);
                for (i, &p) in subpats.iter().enumerate() {
                    let exp = variant_ty.get(i).copied().unwrap_or_else(|| self.db.error());
                    let t = self.check_pat(p, exp);
                    self.pat_types.insert(p, t);
                }
                enum_ty
            }
            PatKind::Or(pats) => {
                let mut ty = self.db.error();
                for &p in &pats {
                    let t = self.check_pat(p, expected);
                    if self.db.is_error(ty) {
                        ty = t;
                    } else {
                        let _ = self.infer.unify(&mut self.db, self.handler, ty, t, span);
                    }
                }
                ty
            }
        };
        self.pat_types.insert(pat, ty);
        ty
    }

    pub fn struct_field_type(&mut self, struct_ty: TypeId, field: &str) -> TypeId {
        if let TyKind::Adt { def, .. } = self.db.kind(struct_ty) {
            if let Some(DefKind::Struct { fields, .. }) = self.resolution.def_map.get(*def) {
                if fields.iter().any(|f| f.as_str() == field) {
                    return self.wk.int;
                }
            }
        }
        self.db.error()
    }

    fn enum_variant_types(&mut self, enum_ty: TypeId, variant: &str) -> Vec<TypeId> {
        if let TyKind::Adt { def, .. } = self.db.kind(enum_ty) {
            if let Some(DefKind::Enum { variants, .. }) = self.resolution.def_map.get(*def) {
                if variants.iter().any(|v| v.as_str() == variant) {
                    return vec![self.wk.int];
                }
            }
        }
        vec![]
    }
}
