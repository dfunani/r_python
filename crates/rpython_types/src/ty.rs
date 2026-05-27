use crate::infer::InferVar;
use crate::subst::Subst;
use crate::trait_ref::TraitRef;
use rpython_ids::{DefId, TypeId};
use rustc_hash::FxHashMap;

/// Integer width for fixed-width integers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntWidth {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl IntWidth {
    pub fn default_int() -> Self {
        Self::I64
    }

    pub fn bytes(self) -> u64 {
        match self {
            Self::I8 | Self::U8 => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 => 4,
            Self::I64 | Self::U64 => 8,
        }
    }
}

/// Float width.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FloatWidth {
    F32,
    F64,
}

impl FloatWidth {
    pub fn bytes(self) -> u64 {
        match self {
            Self::F32 => 4,
            Self::F64 => 8,
        }
    }
}

/// Reference mutability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mutability {
    Imm,
    Mut,
}

/// Region placeholder (lexical regions in v1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct RegionId(pub u32);

/// Function signature in the type system.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FnSig {
    pub params: Vec<TypeId>,
    pub ret: TypeId,
}

/// Canonical type kinds (see Appendix E).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TyKind {
    Bool,
    Int(IntWidth),
    Float(FloatWidth),
    Char,
    Str,
    Bytes,
    Unit,
    Never,
    Tuple(Vec<TypeId>),
    Array {
        elem: TypeId,
        len: usize,
    },
    Slice {
        elem: TypeId,
    },
    Ref {
        mutability: Mutability,
        elem: TypeId,
        region: RegionId,
    },
    Adt {
        def: DefId,
        subst: Subst,
    },
    FnDef {
        def: DefId,
        subst: Subst,
    },
    FnPtr {
        sig: FnSig,
    },
    TraitObject {
        trait_ref: TraitRef,
    },
    Infer(InferVar),
    Error,
    GenericParam {
        index: u32,
    },
}

/// Interned type storage.
#[derive(Clone, Debug, Default)]
pub struct TypeDatabase {
    kinds: Vec<TyKind>,
    interner: FxHashMap<TyKind, TypeId>,
    next_infer: u32,
}

impl TypeDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, kind: TyKind) -> TypeId {
        if let Some(&id) = self.interner.get(&kind) {
            return id;
        }
        let id = TypeId::from_usize(self.kinds.len());
        self.kinds.push(kind.clone());
        self.interner.insert(kind, id);
        id
    }

    pub fn kind(&self, id: TypeId) -> &TyKind {
        &self.kinds[id.index()]
    }

    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }

    pub fn bool(&mut self) -> TypeId {
        self.intern(TyKind::Bool)
    }

    pub fn int(&mut self) -> TypeId {
        self.intern(TyKind::Int(IntWidth::default_int()))
    }

    pub fn str(&mut self) -> TypeId {
        self.intern(TyKind::Str)
    }

    pub fn float(&mut self) -> TypeId {
        self.intern(TyKind::Float(FloatWidth::F64))
    }

    pub fn unit(&mut self) -> TypeId {
        self.intern(TyKind::Unit)
    }

    pub fn never(&mut self) -> TypeId {
        self.intern(TyKind::Never)
    }

    pub fn error(&mut self) -> TypeId {
        self.intern(TyKind::Error)
    }

    pub fn generic_param(&mut self, index: u32) -> TypeId {
        self.intern(TyKind::GenericParam { index })
    }

    pub fn tuple(&mut self, elems: Vec<TypeId>) -> TypeId {
        self.intern(TyKind::Tuple(elems))
    }

    pub fn adt(&mut self, def: DefId, subst: Subst) -> TypeId {
        self.intern(TyKind::Adt { def, subst })
    }

    pub fn fn_def(&mut self, def: DefId, subst: Subst) -> TypeId {
        self.intern(TyKind::FnDef { def, subst })
    }

    pub fn fresh_infer(&mut self) -> TypeId {
        let var = InferVar::new(self.next_infer);
        self.next_infer += 1;
        self.intern(TyKind::Infer(var))
    }

    pub fn is_error(&self, ty: TypeId) -> bool {
        matches!(self.kind(ty), TyKind::Error)
    }

    pub fn is_never(&self, ty: TypeId) -> bool {
        matches!(self.kind(ty), TyKind::Never)
    }

    pub fn is_unit(&self, ty: TypeId) -> bool {
        matches!(self.kind(ty), TyKind::Unit)
    }

    /// Apply substitution throughout `ty`.
    pub fn substitute(&mut self, ty: TypeId, subst: &Subst) -> TypeId {
        match self.kind(ty).clone() {
            TyKind::GenericParam { index } => subst
                .args
                .get(index as usize)
                .copied()
                .unwrap_or_else(|| self.error()),
            TyKind::Tuple(elems) => {
                let mapped: Vec<_> = elems
                    .into_iter()
                    .map(|t| self.substitute(t, subst))
                    .collect();
                self.tuple(mapped)
            }
            TyKind::Array { elem, len } => {
                let elem = self.substitute(elem, subst);
                self.intern(TyKind::Array { elem, len })
            }
            TyKind::Slice { elem } => {
                let elem = self.substitute(elem, subst);
                self.intern(TyKind::Slice { elem })
            }
            TyKind::Ref {
                mutability,
                elem,
                region,
            } => {
                let elem = self.substitute(elem, subst);
                self.intern(TyKind::Ref {
                    mutability,
                    elem,
                    region,
                })
            }
            TyKind::Adt { def, subst: s } => {
                let new_args: Vec<_> = s.args.iter().map(|&t| self.substitute(t, subst)).collect();
                self.adt(def, Subst::from_args(new_args))
            }
            TyKind::FnDef { def, subst: s } => {
                let new_args: Vec<_> = s.args.iter().map(|&t| self.substitute(t, subst)).collect();
                self.fn_def(def, Subst::from_args(new_args))
            }
            TyKind::FnPtr { sig } => {
                let params: Vec<_> = sig
                    .params
                    .into_iter()
                    .map(|p| self.substitute(p, subst))
                    .collect();
                let ret = self.substitute(sig.ret, subst);
                self.intern(TyKind::FnPtr {
                    sig: FnSig { params, ret },
                })
            }
            TyKind::Infer(_) => ty,
            other => self.intern(other),
        }
    }
}
