use crate::def_map::{DefKind, DefMap};
use crate::ribs::RibStack;
use crate::scope::ScopeKind;
use rpython_ast::{Arena, GenericParam, ImplItem, ItemId, ItemKind, Module};
use rpython_errors::{Diagnostic, ErrorCode, Handler};
use rpython_ids::DefId;
use smol_str::SmolStr;

/// First pass: collect top-level and nested definitions without resolving bodies.
pub struct Collector<'a> {
    pub def_map: &'a mut DefMap,
    pub ribs: &'a mut RibStack,
    pub handler: &'a mut Handler,
    pub parent: DefId,
}

impl<'a> Collector<'a> {
    pub fn collect_module(&mut self, module: &Module, arena: &Arena) {
        self.ribs.push(ScopeKind::Root, self.parent, None);
        for &item in &module.items {
            self.collect_item(item, arena);
        }
        self.ribs.pop();
    }

    fn collect_item(&mut self, id: ItemId, arena: &Arena) {
        let item = arena.item(id);
        match &item.kind {
            ItemKind::Function {
                name,
                generics,
                params,
                ret_ty: _,
                body: _,
                is_pub: _,
                attrs: _,
            } => {
                let def = self.def_map.alloc(DefKind::Function {
                    parent: self.parent,
                    name: name.clone(),
                    sig_span: item.span,
                });
                if self.define_name(name.clone(), def, item.span) {
                    return;
                }
                self.def_map.insert_name(self.parent, name.clone(), def);
                self.collect_fn_generics_and_params(def, generics, params, arena);
            }
            ItemKind::Struct {
                name,
                generics: _,
                fields,
                is_pub: _,
                attrs: _,
            } => {
                let field_names: Vec<SmolStr> =
                    fields.iter().map(|f| f.name.clone()).collect();
                let def = self.def_map.alloc(DefKind::Struct {
                    name: name.clone(),
                    fields: field_names,
                });
                if self.define_name(name.clone(), def, item.span) {
                    return;
                }
                self.def_map.insert_name(self.parent, name.clone(), def);
            }
            ItemKind::Enum {
                name,
                generics: _,
                variants,
                is_pub: _,
                attrs: _,
            } => {
                let variant_names: Vec<SmolStr> =
                    variants.iter().map(|v| v.name.clone()).collect();
                let def = self.def_map.alloc(DefKind::Enum {
                    name: name.clone(),
                    variants: variant_names,
                });
                if self.define_name(name.clone(), def, item.span) {
                    return;
                }
                self.def_map.insert_name(self.parent, name.clone(), def);
                for (idx, variant) in variants.iter().enumerate() {
                    let vdef = self.def_map.alloc(DefKind::Variant {
                        parent: def,
                        name: variant.name.clone(),
                        index: idx as u32,
                    });
                    self.def_map
                        .insert_name(def, variant.name.clone(), vdef);
                }
            }
            ItemKind::Trait { name, .. } => {
                let def = self.def_map.alloc(DefKind::Trait {
                    name: name.clone(),
                });
                if self.define_name(name.clone(), def, item.span) {
                    return;
                }
                self.def_map.insert_name(self.parent, name.clone(), def);
            }
            ItemKind::Impl {
                trait_ref,
                self_ty,
                items,
                ..
            } => {
                let trait_name = trait_ref
                    .as_ref()
                    .and_then(|p| p.segments.last())
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default();
                let self_name = type_name_from_ty(*self_ty, arena);
                let def = self.def_map.alloc(DefKind::Impl {
                    trait_ref: None,
                    self_ty_name: self_name.clone().into(),
                });
                self.ribs.push(ScopeKind::Impl, def, Some(self.parent));
                for impl_item in items {
                    self.collect_impl_item(impl_item, def, arena);
                }
                self.ribs.pop();
                let _ = trait_name;
            }
            ItemKind::Const { name, ty: _, value: _, is_pub: _ } => {
                let def = self.def_map.alloc(DefKind::Const {
                    name: name.clone(),
                    ty_span: item.span,
                });
                if self.define_name(name.clone(), def, item.span) {
                    return;
                }
                self.def_map.insert_name(self.parent, name.clone(), def);
            }
            ItemKind::Import { path, alias } => {
                let alias_name = alias
                    .clone()
                    .or_else(|| path.segments.last().map(|s| s.ident.clone()))
                    .unwrap_or_else(|| "unknown".into());
                let path_str: SmolStr = path
                    .segments
                    .iter()
                    .map(|s| s.ident.as_str())
                    .collect::<Vec<_>>()
                    .join(".")
                    .into();
                let def = self.def_map.alloc(DefKind::Import {
                    path: path_str,
                    alias: alias_name.clone(),
                });
                let _ = self.define_name(alias_name, def, item.span);
            }
            ItemKind::Class { name, body, .. } => {
                let def = self.def_map.alloc(DefKind::Struct {
                    name: name.clone(),
                    fields: Vec::new(),
                });
                if self.define_name(name.clone(), def, item.span) {
                    return;
                }
                self.def_map.insert_name(self.parent, name.clone(), def);
                self.ribs.push(ScopeKind::Class, def, Some(self.parent));
                for &nested in body {
                    self.collect_item(nested, arena);
                }
                self.ribs.pop();
            }
            ItemKind::Module { name, items } => {
                let def = self.def_map.alloc(DefKind::Module(
                    rpython_ids::ModuleId(self.def_map.root_module().0),
                ));
                if self.define_name(name.clone(), def, item.span) {
                    return;
                }
                self.ribs.push(ScopeKind::Module, def, Some(self.parent));
                for &nested in items {
                    self.collect_item(nested, arena);
                }
                self.ribs.pop();
            }
            ItemKind::ExternBlock { .. } => {}
        }
    }

    fn collect_impl_item(&mut self, item: &ImplItem, owner: DefId, arena: &Arena) {
        match item {
            ImplItem::Function {
                name,
                generics,
                params,
                ret_ty: _,
                body: _,
                span,
            } => {
                let def = self.def_map.alloc(DefKind::Function {
                    parent: owner,
                    name: name.clone(),
                    sig_span: *span,
                });
                if self.define_name(name.clone(), def, *span) {
                    return;
                }
                self.def_map.insert_name(owner, name.clone(), def);
                self.collect_fn_generics_and_params(def, generics, params, arena);
            }
            ImplItem::Const { name, .. } => {
                let def = self.def_map.alloc(DefKind::Const {
                    name: name.clone(),
                    ty_span: item.span(),
                });
                let _ = self.define_name(name.clone(), def, item.span());
            }
            ImplItem::Type { name, .. } => {
                let def = self.def_map.alloc(DefKind::TypeAlias {
                    name: name.clone(),
                });
                let _ = self.define_name(name.clone(), def, item.span());
            }
        }
    }

    fn collect_fn_generics_and_params(
        &mut self,
        owner: DefId,
        generics: &[GenericParam],
        params: &[rpython_ast::Param],
        _arena: &Arena,
    ) {
        self.ribs.push(ScopeKind::Function, owner, Some(self.parent));
        for (idx, gp) in generics.iter().enumerate() {
            let def = self.def_map.alloc(DefKind::TypeAlias {
                name: gp.name.clone(),
            });
            let _ = self.ribs.define(gp.name.clone(), def);
            let _ = idx;
        }
        for (idx, param) in params.iter().enumerate() {
            let def = self.def_map.alloc(DefKind::Param {
                owner,
                index: idx as u32,
                name: param.name.clone(),
            });
            let _ = self.ribs.define(param.name.clone(), def);
        }
        self.ribs.pop();
    }

    fn define_name(&mut self, name: SmolStr, def: DefId, span: rpython_span::Span) -> bool {
        if let Some(prev) = self.ribs.define(name.clone(), def) {
            self.handler.emit(
                Diagnostic::error(format!("duplicate definition of `{name}`"))
                    .with_code(ErrorCode::E0201)
                    .with_label(span, "duplicate definition", true)
                    .with_label(
                        span,
                        format!("previous definition is also named `{name}`"),
                        false,
                    ),
            );
            let _ = prev;
            return true;
        }
        false
    }
}

fn type_name_from_ty(ty: rpython_ids::TyId, arena: &Arena) -> String {
    match &arena.ty(ty).kind {
        rpython_ast::TyKind::Path(p) => p
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_else(|| "_".into()),
        _ => "_".into(),
    }
}

trait ImplItemSpan {
    fn span(&self) -> rpython_span::Span;
}

impl ImplItemSpan for ImplItem {
    fn span(&self) -> rpython_span::Span {
        match self {
            ImplItem::Function { span, .. }
            | ImplItem::Const { span, .. }
            | ImplItem::Type { span, .. } => *span,
        }
    }
}
