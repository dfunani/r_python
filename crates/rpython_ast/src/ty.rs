use crate::{Path, TyId};
use rpython_span::Span;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A type annotation in the AST.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ty {
    pub kind: TyKind,
    pub span: Span,
}

/// Syntax-level type kinds (see Appendix B.5).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TyKind {
    Path(Path),
    Tuple(Vec<TyId>),
    Array { elem: TyId, len: Option<u64> },
    Slice { elem: TyId },
    Ref { mutability: Mutability, inner: TyId },
    Fn { params: Vec<TyId>, ret: Option<TyId> },
    GenericParam { name: SmolStr },
}

/// Reference mutability for `&T` / `&mut T`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mutability {
    Imm,
    Mut,
}
