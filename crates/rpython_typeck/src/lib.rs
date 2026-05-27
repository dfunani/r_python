//! Type checking for rPython.

mod check_expr;
mod check_fn;
mod check_pat;
mod traits;
mod unify;
mod well_known;

pub use traits::{FulfilledObligation, ImplTable, MonoInstance};
pub use unify::InferCtxt;
pub use well_known::WellKnown;

use rpython_ast::{Arena, Module};
use rpython_errors::Handler;
use rpython_ids::{DefId, ExprId, PatId, TypeId};
use rpython_resolve::{resolve_for_typeck, ResolvedCrate};
use rpython_types::TypeDatabase;
use rustc_hash::FxHashMap;

/// Type-checked crate ready for HIR/MIR lowering.
#[derive(Clone, Debug)]
pub struct TypedCrate {
    pub resolved: ResolvedCrate,
    pub types: TypeDatabase,
    pub unit: TypeId,
    pub int: TypeId,
    pub bool: TypeId,
    pub str: TypeId,
    pub expr_types: FxHashMap<ExprId, TypeId>,
    pub pat_types: FxHashMap<PatId, TypeId>,
    pub item_sigs: FxHashMap<DefId, TypeId>,
    pub fn_ret: FxHashMap<DefId, TypeId>,
    pub fn_params: FxHashMap<DefId, Vec<TypeId>>,
    pub mono_instances: Vec<MonoInstance>,
    pub impl_table: ImplTable,
}

impl TypedCrate {
    pub fn interner(&self) -> &TypeDatabase {
        &self.types
    }
}

/// Type checking context (one compilation unit).
pub struct TypeCtxt<'a> {
    pub resolution: &'a ResolvedCrate,
    pub arena: &'a Arena,
    pub handler: &'a mut Handler,
    pub db: TypeDatabase,
    pub wk: WellKnown,
    pub root: DefId,
    pub infer: InferCtxt,
    pub expr_types: FxHashMap<ExprId, TypeId>,
    pub pat_types: FxHashMap<PatId, TypeId>,
    pub item_sigs: FxHashMap<DefId, TypeId>,
    pub local_types: FxHashMap<DefId, TypeId>,
    pub impl_table: ImplTable,
    pub return_ty: TypeId,
    pub return_checked: bool,
    pub current_fn: Option<DefId>,
    pub mono_instances: Vec<MonoInstance>,
}

impl<'a> TypeCtxt<'a> {
    fn new(
        resolution: &'a ResolvedCrate,
        arena: &'a Arena,
        handler: &'a mut Handler,
    ) -> Self {
        let mut db = TypeDatabase::new();
        let wk = WellKnown::new(&mut db, &resolution.builtins);
        let unit = wk.unit;
        Self {
            resolution,
            arena,
            handler,
            db,
            wk,
            root: resolution.root,
            infer: InferCtxt::new(),
            expr_types: FxHashMap::default(),
            pat_types: FxHashMap::default(),
            item_sigs: FxHashMap::default(),
            local_types: FxHashMap::default(),
            impl_table: ImplTable::new(),
            return_ty: unit,
            return_checked: false,
            current_fn: None,
            mono_instances: Vec::new(),
        }
    }

    fn into_typed(mut self) -> TypedCrate {
        let mut fn_ret = FxHashMap::default();
        let mut fn_params = FxHashMap::default();
        for &(item_id, def_id) in &self.resolution.item_def_ids {
            let item = self.arena.item(item_id);
            if let rpython_ast::ItemKind::Function {
                params, ret_ty, ..
            } = &item.kind
            {
                let mut param_tys = Vec::new();
                for p in params {
                    let ty = p
                        .ty
                        .map(|t| self.ast_ty_to_type(t))
                        .unwrap_or(self.wk.int);
                    param_tys.push(ty);
                }
                let ret = ret_ty
                    .map(|t| self.ast_ty_to_type(t))
                    .unwrap_or(self.wk.unit);
                fn_params.insert(def_id, param_tys);
                fn_ret.insert(def_id, ret);
            }
        }
        TypedCrate {
            resolved: self.resolution.clone(),
            types: self.db,
            unit: self.wk.unit,
            int: self.wk.int,
            bool: self.wk.bool,
            str: self.wk.str,
            expr_types: self.expr_types,
            pat_types: self.pat_types,
            item_sigs: self.item_sigs,
            fn_ret,
            fn_params,
            mono_instances: self.mono_instances,
            impl_table: self.impl_table,
        }
    }
}

/// Run type checking; returns `None` if errors were emitted.
pub fn typeck(
    module: &Module,
    arena: &Arena,
    handler: &mut Handler,
) -> Option<TypedCrate> {
    let resolved = resolve_for_typeck(module, arena, handler)?;
    let mut tcx = TypeCtxt::new(&resolved, arena, handler);
    tcx.collect_item_sigs(arena);
    tcx.check_module_items(&module.items);
    let typed = tcx.into_typed();
    if handler.has_errors() {
        return None;
    }
    Some(typed)
}

/// Alias used by the driver.
pub fn typecheck(
    resolution: &rpython_resolve::Resolution,
    module: &Module,
    arena: &Arena,
    handler: &mut Handler,
) -> Option<TypedCrate> {
    let resolved = ResolvedCrate::from_resolution(resolution.clone(), module, arena);
    let mut tcx = TypeCtxt::new(&resolved, arena, handler);
    tcx.collect_item_sigs(arena);
    tcx.check_module_items(&module.items);
    let typed = tcx.into_typed();
    if handler.has_errors() {
        return None;
    }
    Some(typed)
}

pub type TypedProgram = TypedCrate;
