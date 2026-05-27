use crate::{
    Expr, ExprId, ExprKind, Item, ItemId, ItemKind, Pat, PatId, PatKind, Stmt, StmtId, StmtKind,
    Ty, TyId, TyKind,
};
use rpython_span::Span;

/// Arena-backed storage for all AST nodes in a compilation unit.
#[derive(Clone, Debug, Default)]
pub struct Arena {
    exprs: Vec<Expr>,
    stmts: Vec<Stmt>,
    items: Vec<Item>,
    pats: Vec<Pat>,
    tys: Vec<Ty>,
}

impl Arena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expr_count(&self) -> usize {
        self.exprs.len()
    }

    pub fn stmt_count(&self) -> usize {
        self.stmts.len()
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn pat_count(&self) -> usize {
        self.pats.len()
    }

    pub fn ty_count(&self) -> usize {
        self.tys.len()
    }

    pub fn expr(&self, id: ExprId) -> &Expr {
        &self.exprs[id.index()]
    }

    pub fn expr_mut(&mut self, id: ExprId) -> &mut Expr {
        &mut self.exprs[id.index()]
    }

    pub fn stmt(&self, id: StmtId) -> &Stmt {
        &self.stmts[id.index()]
    }

    pub fn stmt_mut(&mut self, id: StmtId) -> &mut Stmt {
        &mut self.stmts[id.index()]
    }

    pub fn item(&self, id: ItemId) -> &Item {
        &self.items[id.index()]
    }

    pub fn item_mut(&mut self, id: ItemId) -> &mut Item {
        &mut self.items[id.index()]
    }

    pub fn pat(&self, id: PatId) -> &Pat {
        &self.pats[id.index()]
    }

    pub fn pat_mut(&mut self, id: PatId) -> &mut Pat {
        &mut self.pats[id.index()]
    }

    pub fn ty(&self, id: TyId) -> &Ty {
        &self.tys[id.index()]
    }

    pub fn ty_mut(&mut self, id: TyId) -> &mut Ty {
        &mut self.tys[id.index()]
    }

    pub fn alloc_expr(&mut self, kind: ExprKind, span: Span) -> ExprId {
        let id = ExprId::from_usize(self.exprs.len());
        self.exprs.push(Expr { kind, span });
        id
    }

    pub fn alloc_stmt(&mut self, kind: StmtKind, span: Span) -> StmtId {
        let id = StmtId::from_usize(self.stmts.len());
        self.stmts.push(Stmt { kind, span });
        id
    }

    pub fn alloc_item(&mut self, kind: ItemKind, span: Span) -> ItemId {
        let id = ItemId::from_usize(self.items.len());
        self.items.push(Item { kind, span });
        id
    }

    pub fn alloc_pat(&mut self, kind: PatKind, span: Span) -> PatId {
        let id = PatId::from_usize(self.pats.len());
        self.pats.push(Pat { kind, span });
        id
    }

    pub fn alloc_ty(&mut self, kind: TyKind, span: Span) -> TyId {
        let id = TyId::from_usize(self.tys.len());
        self.tys.push(Ty { kind, span });
        id
    }
}
