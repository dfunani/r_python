use rpython_errors::{Diagnostic, ErrorCode, Handler};
use rpython_ids::TypeId;
use rpython_span::Span;
use rpython_types::{InferVar, TyKind, TypeDatabase};
use rustc_hash::FxHashMap;

/// Unification table mapping inference variables to types.
#[derive(Clone, Debug, Default)]
pub struct InferCtxt {
    subst: FxHashMap<InferVar, TypeId>,
}

impl InferCtxt {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve(&mut self, db: &mut TypeDatabase, ty: TypeId) -> TypeId {
        let mut current = ty;
        let mut visited = Vec::new();
        loop {
            match db.kind(current) {
                TyKind::Infer(var) => {
                    if let Some(&to) = self.subst.get(var) {
                        if visited.contains(&to) {
                            return db.error();
                        }
                        visited.push(current);
                        current = to;
                    } else {
                        return current;
                    }
                }
                _ => return current,
            }
        }
    }

    pub fn unify(
        &mut self,
        db: &mut TypeDatabase,
        handler: &mut Handler,
        expected: TypeId,
        found: TypeId,
        span: Span,
    ) -> bool {
        let expected = self.resolve(db, expected);
        let found = self.resolve(db, found);
        if expected == found {
            return true;
        }
        match (db.kind(expected).clone(), db.kind(found).clone()) {
            (TyKind::Infer(var), _) => {
                self.subst.insert(var, found);
                true
            }
            (_, TyKind::Infer(var)) => {
                self.subst.insert(var, expected);
                true
            }
            (TyKind::Tuple(a), TyKind::Tuple(b)) if a.len() == b.len() => {
                let mut ok = true;
                for (&ae, &be) in a.iter().zip(b.iter()) {
                    ok &= self.unify(db, handler, ae, be, span);
                }
                ok
            }
            (TyKind::Adt { def: d1, subst: s1 }, TyKind::Adt { def: d2, subst: s2 })
                if d1 == d2 && s1.args.len() == s2.args.len() =>
            {
                let mut ok = true;
                for (&a, &b) in s1.args.iter().zip(s2.args.iter()) {
                    ok &= self.unify(db, handler, a, b, span);
                }
                ok
            }
            (TyKind::Error, _) | (_, TyKind::Error) => true,
            (TyKind::Never, _) | (_, TyKind::Never) => true,
            (e, f) => {
                handler.emit(
                    Diagnostic::error(format!(
                        "type mismatch: expected `{}`, found `{}`",
                        type_name(db, expected, self),
                        type_name(db, found, self)
                    ))
                    .with_code(ErrorCode::E0300)
                    .with_label(span, "type mismatch here", true),
                );
                let _ = (e, f);
                false
            }
        }
    }

    pub fn apply_to_db(&self, db: &mut TypeDatabase, ty: TypeId) -> TypeId {
        match db.kind(ty).clone() {
            TyKind::Infer(var) => {
                if let Some(&to) = self.subst.get(&var) {
                    self.apply_to_db(db, to)
                } else {
                    ty
                }
            }
            TyKind::Tuple(elems) => {
                let mapped: Vec<_> = elems.into_iter().map(|t| self.apply_to_db(db, t)).collect();
                db.tuple(mapped)
            }
            other => db.intern(other),
        }
    }
}

pub fn type_name(db: &mut TypeDatabase, ty: TypeId, infer: &InferCtxt) -> String {
    let mut cx = InferCtxt {
        subst: infer.subst.clone(),
    };
    let ty = cx.resolve(db, ty);
    let kind = db.kind(ty).clone();
    match kind {
        TyKind::Bool => "bool".into(),
        TyKind::Int(_) => "int".into(),
        TyKind::Float(_) => "float".into(),
        TyKind::Str => "str".into(),
        TyKind::Unit => "void".into(),
        TyKind::Never => "!".into(),
        TyKind::Tuple(elems) => {
            let inner: Vec<_> = elems.iter().map(|t| type_name(db, *t, infer)).collect();
            format!("({})", inner.join(", "))
        }
        TyKind::Adt { def, .. } => format!("Adt({})", def.0),
        TyKind::FnDef { def, .. } => format!("fn({})", def.0),
        TyKind::Infer(v) => format!("?{}", v.0),
        TyKind::Error => "<error>".into(),
        TyKind::GenericParam { index } => format!("T{index}"),
        other => format!("{other:?}"),
    }
}
