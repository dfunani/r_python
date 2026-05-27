use crate::body::{BasicBlock, MirBody};
use crate::statement::Statement;
use crate::terminator::Terminator;

pub trait MirVisitor {
    fn visit_body(&mut self, body: &MirBody) {
        visit_body(self, body);
    }
    fn visit_block(&mut self, _bb: usize, _block: &BasicBlock) {}
    fn visit_statement(&mut self, _stmt: &Statement) {}
    fn visit_terminator(&mut self, _term: &Terminator) {}
}

pub fn visit_body<V: MirVisitor + ?Sized>(v: &mut V, body: &MirBody) {
    for (i, block) in body.blocks.iter().enumerate() {
        v.visit_block(i, block);
        for stmt in &block.statements {
            v.visit_statement(stmt);
        }
        v.visit_terminator(&block.terminator);
    }
}
