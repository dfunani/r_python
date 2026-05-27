use rpython_ids::{DefId, ModuleId};
use rpython_span::Span;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

#[derive(Clone, Debug)]
pub enum DefKind {
    Module(ModuleId),
    Function {
        parent: DefId,
        name: SmolStr,
        sig_span: Span,
    },
    Struct {
        name: SmolStr,
        fields: Vec<SmolStr>,
    },
    Enum {
        name: SmolStr,
        variants: Vec<SmolStr>,
    },
    Variant {
        parent: DefId,
        name: SmolStr,
        index: u32,
    },
    Interface {
        name: SmolStr,
    },
    Impl {
        interface_ref: Option<DefId>,
        self_ty_name: SmolStr,
    },
    Const {
        name: SmolStr,
        ty_span: Span,
    },
    TypeAlias {
        name: SmolStr,
    },
    Import {
        path: SmolStr,
        alias: SmolStr,
    },
    Param {
        owner: DefId,
        index: u32,
        name: SmolStr,
    },
    Local {
        owner: DefId,
        index: u32,
        name: SmolStr,
    },
    BuiltinType {
        name: SmolStr,
    },
    BuiltinFn {
        name: SmolStr,
    },
    ExternFn {
        name: SmolStr,
    },
    ExternBlock,
}

#[derive(Clone, Debug)]
struct DefEntry {
    kind: DefKind,
}

#[derive(Clone, Debug, Default)]
pub struct DefMap {
    defs: Vec<DefEntry>,
    children: FxHashMap<DefId, FxHashMap<SmolStr, DefId>>,
    root_module: ModuleId,
    root_def: DefId,
}

impl DefMap {
    pub fn new(root_module: ModuleId) -> Self {
        let mut map = Self {
            defs: Vec::new(),
            children: FxHashMap::default(),
            root_module,
            root_def: DefId(0),
        };
        let root = map.alloc(DefKind::Module(root_module));
        map.root_def = root;
        map
    }

    pub fn root_module(&self) -> ModuleId {
        self.root_module
    }

    pub fn root_def(&self) -> DefId {
        self.root_def
    }

    pub fn alloc(&mut self, kind: DefKind) -> DefId {
        let id = DefId::from_usize(self.defs.len());
        self.defs.push(DefEntry { kind });
        id
    }

    pub fn kind(&self, id: DefId) -> &DefKind {
        &self.defs[id.index()].kind
    }

    pub fn insert_name(&mut self, parent: DefId, name: SmolStr, def: DefId) {
        self.children
            .entry(parent)
            .or_default()
            .insert(name, def);
    }

    pub fn lookup(&self, parent: DefId, name: &str) -> Option<DefId> {
        self.children
            .get(&parent)?
            .get(name)
            .copied()
    }

    pub fn children_of(&self, parent: DefId) -> impl Iterator<Item = (&SmolStr, DefId)> + '_ {
        self.children
            .get(&parent)
            .into_iter()
            .flat_map(|m| m.iter().map(|(k, v)| (k, *v)))
    }

    pub fn get(&self, id: DefId) -> Option<&DefKind> {
        self.defs.get(id.index()).map(|e| &e.kind)
    }

    pub fn iter(&self) -> impl Iterator<Item = (DefId, &DefKind)> {
        self.defs
            .iter()
            .enumerate()
            .map(|(i, e)| (DefId::from_usize(i), &e.kind))
    }

    pub fn name(&self, id: DefId) -> Option<SmolStr> {
        match self.get(id)? {
            DefKind::Function { name, .. }
            | DefKind::Struct { name, .. }
            | DefKind::Enum { name, .. }
            | DefKind::Interface { name }
            | DefKind::Const { name, .. }
            | DefKind::TypeAlias { name }
            | DefKind::BuiltinType { name }
            | DefKind::BuiltinFn { name }
            | DefKind::ExternFn { name }
            | DefKind::Param { name, .. }
            | DefKind::Local { name, .. }
            | DefKind::Variant { name, .. } => Some(name.clone()),
            DefKind::Import { alias, .. } => Some(alias.clone()),
            DefKind::Impl { self_ty_name, .. } => Some(self_ty_name.clone()),
            DefKind::Module(_) | DefKind::ExternBlock => None,
        }
    }
}
