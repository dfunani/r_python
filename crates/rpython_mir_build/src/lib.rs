//! Lower HIR to MIR.

mod builder;
mod lower;

use rpython_hir::{HirCrate, HirOwnerKind};
use rpython_ids::MirFuncId;
use rpython_mir::MirCrate;

pub use lower::lower_function;

/// Build MIR for all functions in a HIR crate.
pub fn build_mir(hir: &HirCrate) -> MirCrate {
    let mut crate_ = MirCrate::default();
    for (&def_id, owner) in &hir.owners {
        let HirOwnerKind::Function(body) = &owner.kind;
        let func_id = MirFuncId(crate_.functions.len() as u32);
        let mir_body = lower::lower_function(body);
        crate_.def_to_func.insert(def_id, func_id);
        crate_.functions.insert(func_id, mir_body);
    }
    crate_
}

/// Build MIR from typed AST via HIR (driver convenience).
pub fn build_mir_from_typed(
    typed: &rpython_typeck::TypedCrate,
    module: &rpython_ast::Module,
    arena: &rpython_ast::Arena,
) -> MirCrate {
    let hir = rpython_hir_build::build_hir(typed, module, arena);
    build_mir(&hir)
}
