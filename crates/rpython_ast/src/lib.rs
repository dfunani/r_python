//! Arena-backed abstract syntax tree for rPython.
//!
//! See `docs/IMPLEMENTATION.md` Appendix B for the node catalog.

mod arena;
mod display;
mod expr;
mod item;
mod literal;
mod pat;
mod path;
mod stmt;
mod ty;
mod visit;

pub use arena::Arena;
pub use display::{format_module, AstPrinter, PrettyPrinter};
pub use expr::{
    BinaryOp, Expr, ExprKind, FieldExpr, Kwarg, LambdaParam, Mutability as ExprMutability, UnaryOp,
};
pub use item::{
    Abi, Attribute, ExternItem, FieldDef, GenericParam, ImplItem, InterfaceItem, Item, ItemKind,
    Mutability as ItemMutability, Param, Variant, VariantFields,
};
pub use literal::Literal;
pub use pat::{Mutability as PatMutability, Pat, PatField, PatKind};
pub use path::{Path, PathSegment};
pub use rpython_ids::{ExprId, ItemId, PatId, StmtId, TyId};
pub use rpython_span::Span;
pub use stmt::{ElifArm, Label, MatchArm, Stmt, StmtKind};
pub use ty::{Mutability as TyMutability, Ty, TyKind};
pub use visit::{
    walk_expr, walk_field_defs, walk_generic_params, walk_item, walk_module, walk_params, walk_pat,
    walk_stmt, walk_ty, walk_variants, Visitor,
};

/// Root of a parsed compilation unit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub items: Vec<ItemId>,
    pub span: Span,
}

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;
    use rpython_span::{BytePos, FileId};

    fn span() -> Span {
        Span::new(FileId(0), BytePos(0), BytePos(1))
    }

    #[test]
    fn arena_alloc_and_pretty_print() {
        let mut arena = Arena::new();
        let lit = arena.alloc_expr(ExprKind::Literal(Literal::Int(42)), span());
        let ret = arena.alloc_stmt(StmtKind::Return(Some(lit)), span());
        let _func = arena.alloc_item(
            ItemKind::Function {
                name: "main".into(),
                generics: vec![],
                params: vec![],
                ret_ty: None,
                body: vec![ret],
                is_pub: true,
                attrs: vec![],
            },
            span(),
        );
        let module = Module {
            items: vec![_func],
            span: span(),
        };
        let out = format_module(&module, &arena);
        assert!(out.contains("Function"));
        assert!(out.contains("Return"));
        assert!(out.contains("Literal"));
        assert!(out.contains("42"));
    }

    #[test]
    fn visitor_counts_exprs() {
        struct ExprCounter(usize);

        impl Visitor for ExprCounter {
            fn visit_expr(&mut self, _expr: &Expr, _arena: &Arena) {
                self.0 += 1;
            }
        }

        let mut arena = Arena::new();
        let a = arena.alloc_expr(ExprKind::Literal(Literal::Int(1)), span());
        let b = arena.alloc_expr(ExprKind::Literal(Literal::Int(2)), span());
        let _bin = arena.alloc_expr(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: a,
                right: b,
            },
            span(),
        );
        let module = Module {
            items: vec![],
            span: span(),
        };
        let mut counter = ExprCounter(0);
        walk_module(&mut counter, &module, &arena);
        assert_eq!(counter.0, 0);

        let stmt = arena.alloc_stmt(StmtKind::Expr(_bin), span());
        let _func = arena.alloc_item(
            ItemKind::Function {
                name: "f".into(),
                generics: vec![],
                params: vec![],
                ret_ty: None,
                body: vec![stmt],
                is_pub: false,
                attrs: vec![],
            },
            span(),
        );
        let module = Module {
            items: vec![_func],
            span: span(),
        };
        let mut counter = ExprCounter(0);
        walk_module(&mut counter, &module, &arena);
        assert_eq!(counter.0, 3);
    }
}
