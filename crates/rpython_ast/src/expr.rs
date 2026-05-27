use crate::{ExprId, Literal, Path, StmtId, TyId};
use rpython_span::Span;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// An expression node in the AST.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

/// Expression kinds (see Appendix B.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    Literal(Literal),
    Path(Path),
    Call {
        func: ExprId,
        args: Vec<ExprId>,
        kwargs: Vec<Kwarg>,
    },
    MethodCall {
        receiver: ExprId,
        method: SmolStr,
        args: Vec<ExprId>,
    },
    Field {
        base: ExprId,
        field: SmolStr,
    },
    Index {
        base: ExprId,
        index: ExprId,
    },
    Unary {
        op: UnaryOp,
        operand: ExprId,
    },
    Binary {
        op: BinaryOp,
        left: ExprId,
        right: ExprId,
    },
    Tuple(Vec<ExprId>),
    List(Vec<ExprId>),
    Struct {
        path: Path,
        fields: Vec<FieldExpr>,
    },
    If {
        test: ExprId,
        then: ExprId,
        else_branch: ExprId,
    },
    Block(Vec<StmtId>),
    Lambda {
        params: Vec<LambdaParam>,
        body: ExprId,
    },
    Cast {
        expr: ExprId,
        ty: TyId,
    },
    Ref {
        mutability: Mutability,
        expr: ExprId,
    },
    Deref(ExprId),
}

/// Unary operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Neg,
    Pos,
    BitNot,
}

/// Binary operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    FloorDiv,
    Pow,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Is,
    In,
}

/// Reference mutability for `&expr` / `&mut expr`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mutability {
    Imm,
    Mut,
}

/// A keyword argument in a call (`name=value`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Kwarg {
    pub name: SmolStr,
    pub value: ExprId,
    pub span: Span,
}

/// A field initializer in a struct expression (`field: expr`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldExpr {
    pub name: SmolStr,
    pub expr: ExprId,
    pub span: Span,
}

/// A parameter in a lambda expression.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LambdaParam {
    pub name: SmolStr,
    pub ty: Option<TyId>,
    pub span: Span,
}
