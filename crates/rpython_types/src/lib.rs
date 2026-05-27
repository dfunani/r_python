//! Canonical type representation, interning, substitution, and layout.

mod fold;
mod infer;
mod layout;
mod subst;
mod trait_ref;
mod ty;

pub use fold::{default_fold_ty, TypeFolder};
pub use infer::InferVar;
pub use layout::{layout_of, Align, Layout, Size};
pub use rpython_ids::TypeId;
pub use subst::Subst;
pub use trait_ref::TraitRef;
pub use ty::{FloatWidth, FnSig, IntWidth, Mutability, RegionId, TyKind, TypeDatabase};
