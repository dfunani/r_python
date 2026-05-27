use crate::{ExprId, PatId, StmtId, TyId};
use rpython_span::Span;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A statement node in the AST.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

/// Statement kinds (see Appendix B.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StmtKind {
    Expr(ExprId),
    Assign {
        targets: Vec<PatId>,
        value: ExprId,
    },
    AnnAssign {
        target: PatId,
        ty: TyId,
        value: Option<ExprId>,
    },
    Return(Option<ExprId>),
    Raise(ExprId),
    Assert {
        test: ExprId,
        msg: Option<ExprId>,
    },
    Pass,
    Break(Option<Label>),
    Continue(Option<Label>),
    While {
        test: ExprId,
        body: Vec<StmtId>,
    },
    For {
        pat: PatId,
        iter: ExprId,
        body: Vec<StmtId>,
    },
    If {
        test: ExprId,
        then_body: Vec<StmtId>,
        elifs: Vec<ElifArm>,
        else_body: Option<Vec<StmtId>>,
    },
    Match {
        scrutinee: ExprId,
        arms: Vec<MatchArm>,
    },
}

/// Optional label on `break` / `continue`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Label(pub SmolStr);

/// One `elif` branch in an `if` statement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ElifArm {
    pub test: ExprId,
    pub body: Vec<StmtId>,
    pub span: Span,
}

/// One arm in a `match` statement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pat: PatId,
    pub guard: Option<ExprId>,
    pub body: Vec<StmtId>,
    pub span: Span,
}
