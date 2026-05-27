use rpython_span::Span;

use crate::ids::{HirExprId, HirStmtId};
use crate::place::{Place, Rvalue};

/// HIR statement.
#[derive(Clone, Debug)]
pub struct HirStmt {
    pub kind: HirStmtKind,
    pub span: Span,
}

/// HIR statement kinds.
#[derive(Clone, Debug)]
pub enum HirStmtKind {
    Assign {
        place: Place,
        rvalue: Rvalue,
    },
    Expr(HirExprId),
    Return(Option<HirExprId>),
    Drop(Place),
    While {
        cond: HirExprId,
        body: Vec<HirStmtId>,
    },
}

/// Function body in HIR.
#[derive(Clone, Debug)]
pub struct HirBody {
    pub def_id: rpython_ids::DefId,
    pub name: smol_str::SmolStr,
    pub params: Vec<rpython_ids::LocalId>,
    pub ret_ty: rpython_types::TypeId,
    pub stmts: Vec<HirStmtId>,
    pub exprs: Vec<crate::expr::HirExpr>,
    pub stmts_data: Vec<HirStmt>,
    pub pats: Vec<crate::pat::HirPat>,
    pub locals: Vec<LocalDecl>,
}

/// Local variable declaration in HIR.
#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub ty: rpython_types::TypeId,
    pub mutability: rpython_types::Mutability,
    pub span: rpython_span::Span,
}
