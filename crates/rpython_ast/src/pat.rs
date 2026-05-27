use crate::{Literal, Path, PatId};
use rpython_span::Span;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A pattern in assignments, `for`, `match`, and parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pat {
    pub kind: PatKind,
    pub span: Span,
}

/// Pattern kinds (see Appendix B.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PatKind {
    Wild,
    Ident {
        name: SmolStr,
        mutability: Mutability,
        subpat: Option<PatId>,
    },
    Literal(Literal),
    Tuple(Vec<PatId>),
    Struct {
        path: Path,
        fields: Vec<PatField>,
    },
    Enum {
        path: Path,
        variant: SmolStr,
        subpats: Vec<PatId>,
    },
    Or(Vec<PatId>),
}

/// Mutability on binding patterns (`mut x`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mutability {
    Imm,
    Mut,
}

/// A field in a struct pattern (`Point { x, y: pat }`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PatField {
    pub name: SmolStr,
    pub pat: PatId,
    pub span: Span,
}
