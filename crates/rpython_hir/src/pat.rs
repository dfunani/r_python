use rpython_ids::DefId;
use rpython_span::Span;
use smol_str::SmolStr;

use crate::ids::HirPatId;
use crate::place::HirConst;

/// HIR pattern.
#[derive(Clone, Debug)]
pub struct HirPat {
    pub kind: HirPatKind,
    pub span: Span,
}

/// HIR pattern kinds.
#[derive(Clone, Debug)]
pub enum HirPatKind {
    Wild,
    Literal(HirConst),
    Local {
        name: SmolStr,
        local: rpython_ids::LocalId,
    },
    Enum {
        def: DefId,
        variant: u32,
        subpats: Vec<HirPatId>,
    },
}
