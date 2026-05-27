//! High-level IR: typed, desugared AST close to MIR.

mod expr;
mod ids;
mod owner;
mod pat;
mod place;
mod stmt;

pub use expr::{HirExpr, HirExprKind};
pub use ids::{HirExprId, HirPatId, HirStmtId};
pub use owner::{HirCrate, HirOwner, HirOwnerKind};
pub use pat::{HirPat, HirPatKind};
pub use place::{
    AggregateKind, BinaryOp, HirConst, Operand, Place, Projection, Rvalue, UnaryOp,
};
pub use stmt::{HirBody, HirStmt, HirStmtKind, LocalDecl};
