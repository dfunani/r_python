use crate::subst::Subst;
use crate::ty::{TyKind, TypeDatabase};
use rpython_ids::TypeId;

/// Type folder trait for transforming types.
pub trait TypeFolder {
    fn fold_ty(&mut self, db: &mut TypeDatabase, ty: TypeId) -> TypeId {
        default_fold_ty(self, db, ty)
    }
}

pub fn default_fold_ty<F: TypeFolder + ?Sized>(
    folder: &mut F,
    db: &mut TypeDatabase,
    ty: TypeId,
) -> TypeId {
    let kind = db.kind(ty).clone();
    match kind {
        TyKind::Tuple(elems) => {
            let mapped: Vec<_> = elems.into_iter().map(|t| folder.fold_ty(db, t)).collect();
            db.tuple(mapped)
        }
        TyKind::Array { elem, len } => {
            let elem = folder.fold_ty(db, elem);
            db.intern(TyKind::Array { elem, len })
        }
        TyKind::Slice { elem } => {
            let elem = folder.fold_ty(db, elem);
            db.intern(TyKind::Slice { elem })
        }
        TyKind::Ref {
            mutability,
            elem,
            region,
        } => {
            let elem = folder.fold_ty(db, elem);
            db.intern(TyKind::Ref {
                mutability,
                elem,
                region,
            })
        }
        TyKind::Adt { def, subst } => {
            let args: Vec<_> = subst.args.into_iter().map(|t| folder.fold_ty(db, t)).collect();
            db.adt(def, Subst::from_args(args))
        }
        TyKind::FnDef { def, subst } => {
            let args: Vec<_> = subst.args.into_iter().map(|t| folder.fold_ty(db, t)).collect();
            db.fn_def(def, Subst::from_args(args))
        }
        other => db.intern(other),
    }
}
