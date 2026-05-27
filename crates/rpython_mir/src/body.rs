use indexmap::IndexMap;
use rpython_ids::{DefId, LocalId, MirFuncId, TypeId};
use rpython_span::Span;
use rpython_types::Mutability;
use smol_str::SmolStr;

use crate::statement::Statement;
use crate::terminator::Terminator;

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub ty: TypeId,
    pub mutability: Mutability,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

#[derive(Clone, Debug)]
pub struct SourceScope {
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct MirBody {
    pub name: SmolStr,
    pub def_id: DefId,
    pub arg_count: usize,
    pub return_ty: TypeId,
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<BasicBlock>,
    pub source_scopes: Vec<SourceScope>,
}

#[derive(Clone, Debug, Default)]
pub struct MirCrate {
    pub functions: IndexMap<MirFuncId, MirBody>,
    pub def_to_func: IndexMap<DefId, MirFuncId>,
}

impl MirBody {
    pub fn arg_local(&self, arg_index: usize) -> LocalId {
        LocalId::from_usize(1 + arg_index)
    }

    pub fn local_ty(&self, local: LocalId) -> TypeId {
        self.locals[local.index()].ty
    }
}

impl MirCrate {
    pub fn get_by_def(&self, def: DefId) -> Option<&MirBody> {
        self.def_to_func
            .get(&def)
            .and_then(|id| self.functions.get(id))
    }
}
