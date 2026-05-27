use crate::{ExprId, ItemId, Path, StmtId, TyId};
use rpython_span::Span;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A top-level or nested item (definition).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
}

/// Item kinds (see Appendix B.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ItemKind {
    Function {
        name: SmolStr,
        generics: Vec<GenericParam>,
        params: Vec<Param>,
        ret_ty: Option<TyId>,
        body: Vec<StmtId>,
        is_pub: bool,
        attrs: Vec<Attribute>,
    },
    Class {
        name: SmolStr,
        generics: Vec<GenericParam>,
        bases: Vec<TyId>,
        body: Vec<ItemId>,
        is_pub: bool,
        attrs: Vec<Attribute>,
    },
    Struct {
        name: SmolStr,
        generics: Vec<GenericParam>,
        fields: Vec<FieldDef>,
        is_pub: bool,
        attrs: Vec<Attribute>,
    },
    Enum {
        name: SmolStr,
        generics: Vec<GenericParam>,
        variants: Vec<Variant>,
        is_pub: bool,
        attrs: Vec<Attribute>,
    },
    Interface {
        name: SmolStr,
        generics: Vec<GenericParam>,
        items: Vec<InterfaceItem>,
        is_pub: bool,
        attrs: Vec<Attribute>,
    },
    Impl {
        generics: Vec<GenericParam>,
        interface_ref: Option<Path>,
        self_ty: TyId,
        items: Vec<ImplItem>,
        attrs: Vec<Attribute>,
    },
    Const {
        name: SmolStr,
        ty: TyId,
        value: ExprId,
        is_pub: bool,
    },
    Import {
        path: Path,
        alias: Option<SmolStr>,
    },
    ExternBlock {
        abi: Abi,
        items: Vec<ExternItem>,
    },
    Module {
        name: SmolStr,
        items: Vec<ItemId>,
    },
}

/// Source attribute (`@name` or `@name(args)`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: SmolStr,
    pub args: Vec<ExprId>,
    pub span: Span,
}

/// Generic type parameter (`T`, `T: Bound`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericParam {
    pub name: SmolStr,
    pub bounds: Vec<TyId>,
    pub span: Span,
}

/// Function or method parameter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: SmolStr,
    pub ty: Option<TyId>,
    pub default: Option<ExprId>,
    pub span: Span,
}

/// Struct or record field definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: SmolStr,
    pub ty: TyId,
    pub span: Span,
}

/// Enum variant definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub name: SmolStr,
    pub fields: VariantFields,
    pub span: Span,
}

/// Fields carried by an enum variant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VariantFields {
    Unit,
    Tuple(Vec<TyId>),
    Struct(Vec<FieldDef>),
}

/// Item inside an `interface` block.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InterfaceItem {
    Function {
        name: SmolStr,
        generics: Vec<GenericParam>,
        params: Vec<Param>,
        ret_ty: Option<TyId>,
        default_body: Option<Vec<StmtId>>,
        span: Span,
    },
    Type {
        name: SmolStr,
        ty: Option<TyId>,
        span: Span,
    },
}

/// Item inside an `impl` block.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImplItem {
    Function {
        name: SmolStr,
        generics: Vec<GenericParam>,
        params: Vec<Param>,
        ret_ty: Option<TyId>,
        body: Vec<StmtId>,
        span: Span,
    },
    Const {
        name: SmolStr,
        ty: TyId,
        value: ExprId,
        span: Span,
    },
    Type {
        name: SmolStr,
        ty: TyId,
        span: Span,
    },
}

/// Item inside an `extern` block.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExternItem {
    Function {
        name: SmolStr,
        params: Vec<Param>,
        ret_ty: Option<TyId>,
        span: Span,
    },
    Static {
        name: SmolStr,
        ty: TyId,
        mutability: Mutability,
        span: Span,
    },
}

/// ABI for `extern` blocks.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Abi {
    C,
    RPython,
    Other(SmolStr),
}

/// Mutability for `extern` statics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mutability {
    Imm,
    Mut,
}
