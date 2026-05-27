use rpython_ids::{DefId, LocalId};
use rpython_span::Span;
use rpython_types::{Mutability, Subst, TypeId};

use crate::ids::{HirExprId, HirPatId};
use crate::place::{HirConst, Place};

/// HIR expression.
#[derive(Clone, Debug)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub ty: TypeId,
    pub span: Span,
}

/// HIR expression kinds.
#[derive(Clone, Debug)]
pub enum HirExprKind {
    Literal(HirConst),
    Path {
        def: DefId,
        subst: Subst,
    },
    Local(LocalId),
    Unary {
        op: crate::place::UnaryOp,
        operand: HirExprId,
    },
    Binary {
        op: crate::place::BinaryOp,
        left: HirExprId,
        right: HirExprId,
    },
    Call {
        def: DefId,
        subst: Subst,
        args: Vec<HirExprId>,
    },
    Field {
        base: HirExprId,
        field_index: u32,
    },
    If {
        cond: HirExprId,
        then: HirExprId,
        else_branch: HirExprId,
    },
    Match {
        scrutinee: HirExprId,
        arms: Vec<(HirPatId, HirExprId)>,
    },
    Tuple(Vec<HirExprId>),
    Struct {
        def: DefId,
        fields: Vec<(u32, HirExprId)>,
    },
    AddrOf {
        mutability: Mutability,
        place: Place,
    },
    Deref(Place),
}
