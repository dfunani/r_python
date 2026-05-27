use rpython_ids::{BlockId, LocalId};
use rpython_mir::{
    BasicBlock, BinOp, ConstValue, LocalDecl, Operand, Place, Rvalue, Statement, StatementKind,
    Terminator, TerminatorKind, UnaryOp,
};
use rpython_span::Span;
use rpython_types::{Mutability, TypeId};

/// CFG builder for one MIR function.
pub struct MirBuilder {
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<BasicBlock>,
    pub next_local: usize,
    pub current_block: BlockId,
}

impl MirBuilder {
    pub fn new(ret_ty: TypeId, span: Span) -> Self {
        let mut locals = vec![LocalDecl {
            ty: ret_ty,
            mutability: Mutability::Imm,
            span,
        }];
        let mut builder = Self {
            locals,
            blocks: Vec::new(),
            next_local: 1,
            current_block: BlockId::from_usize(0),
        };
        builder.start_block(span);
        builder
    }

    pub fn alloc_local(&mut self, ty: TypeId, mutability: Mutability, span: Span) -> LocalId {
        let id = LocalId::from_usize(self.next_local);
        self.next_local += 1;
        self.locals.push(LocalDecl {
            ty,
            mutability,
            span,
        });
        id
    }

    pub fn start_block(&mut self, span: Span) {
        let id = BlockId::from_usize(self.blocks.len());
        self.current_block = id;
        self.blocks.push(BasicBlock {
            statements: Vec::new(),
            terminator: Terminator {
                kind: TerminatorKind::Unreachable,
                span,
            },
        });
    }

    pub fn new_block(&mut self, span: Span) -> BlockId {
        let id = BlockId::from_usize(self.blocks.len());
        self.blocks.push(BasicBlock {
            statements: Vec::new(),
            terminator: Terminator {
                kind: TerminatorKind::Unreachable,
                span,
            },
        });
        id
    }

    pub fn switch_to(&mut self, block: BlockId) {
        self.current_block = block;
    }

    pub fn push_stmt(&mut self, kind: StatementKind, span: Span) {
        self.blocks[self.current_block.index()]
            .statements
            .push(Statement { kind, span });
    }

    pub fn assign_const(&mut self, place: Place, val: ConstValue, span: Span) {
        self.push_stmt(
            StatementKind::Assign {
                place,
                rvalue: Rvalue::Use(Operand::Constant(val)),
            },
            span,
        );
    }

    pub fn assign_use(&mut self, place: Place, op: Operand, span: Span) {
        self.push_stmt(
            StatementKind::Assign {
                place,
                rvalue: Rvalue::Use(op),
            },
            span,
        );
    }

    pub fn assign_rvalue(&mut self, place: Place, rvalue: Rvalue, span: Span) {
        self.push_stmt(StatementKind::Assign { place, rvalue }, span);
    }

    pub fn terminate(&mut self, kind: TerminatorKind, span: Span) {
        self.blocks[self.current_block.index()].terminator = Terminator { kind, span };
    }

    pub fn finish(mut self) -> (Vec<LocalDecl>, Vec<BasicBlock>) {
        if matches!(
            self.blocks.last().map(|b| &b.terminator.kind),
            Some(TerminatorKind::Unreachable)
        ) {
            let span = self.blocks.last().unwrap().terminator.span;
            let last = self.blocks.len() - 1;
            self.blocks[last].terminator = Terminator {
                kind: TerminatorKind::Return,
                span,
            };
        }
        (self.locals, self.blocks)
    }
}

pub fn hir_unary_to_mir(op: rpython_hir::UnaryOp) -> UnaryOp {
    match op {
        rpython_hir::UnaryOp::Not => UnaryOp::Not,
        rpython_hir::UnaryOp::Neg => UnaryOp::Neg,
    }
}

pub fn hir_binary_to_mir(op: rpython_hir::BinaryOp) -> BinOp {
    use rpython_hir::BinaryOp as H;
    match op {
        H::Add => BinOp::Add,
        H::Sub => BinOp::Sub,
        H::Mul => BinOp::Mul,
        H::Div => BinOp::Div,
        H::Eq => BinOp::Eq,
        H::NotEq => BinOp::Ne,
        H::Lt => BinOp::Lt,
        H::LtEq => BinOp::Le,
        H::Gt => BinOp::Gt,
        H::GtEq => BinOp::Ge,
        H::And => BinOp::And,
        H::Or => BinOp::Or,
    }
}

pub fn hir_const_to_mir(c: &rpython_hir::HirConst) -> ConstValue {
    match c {
        rpython_hir::HirConst::Int(n) => ConstValue::Int(*n),
        rpython_hir::HirConst::Bool(b) => ConstValue::Bool(*b),
        rpython_hir::HirConst::Float(f) => ConstValue::Float(*f),
        rpython_hir::HirConst::Str(s) => ConstValue::Str(s.to_string()),
        rpython_hir::HirConst::Unit => ConstValue::Unit,
    }
}
