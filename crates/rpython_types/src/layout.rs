use crate::TyKind;
use crate::TypeDatabase;
use rpython_ids::{DefId, TypeId};

/// Byte size of a type in memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Size(pub u64);

/// Alignment requirement of a type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Align(pub u64);

/// Memory layout for codegen and MIR.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub size: Size,
    pub align: Align,
}

impl Layout {
    pub const ZERO: Self = Self {
        size: Size(0),
        align: Align(1),
    };

    pub fn of_primitive(kind: &TyKind) -> Self {
        match kind {
            TyKind::Bool => Self {
                size: Size(1),
                align: Align(1),
            },
            TyKind::Int(_) => Self {
                size: Size(8),
                align: Align(8),
            },
            TyKind::Float(w) => Self {
                size: Size(w.bytes()),
                align: Align(w.bytes()),
            },
            TyKind::Unit | TyKind::Never => Self::ZERO,
            TyKind::Str => Self {
                size: Size(16),
                align: Align(8),
            },
            _ => Self {
                size: Size(8),
                align: Align(8),
            },
        }
    }
}

/// Compute a basic layout for `ty` (no niche optimization).
pub fn layout_of(db: &TypeDatabase, ty: TypeId) -> Layout {
    match db.kind(ty) {
        TyKind::Tuple(elems) => {
            let mut size = 0u64;
            let mut align = 1u64;
            for &e in elems {
                let l = layout_of(db, e);
                let a = l.align.0;
                size = size.div_ceil(a) * a + l.size.0;
                align = align.max(a);
            }
            size = size.div_ceil(align) * align;
            Layout {
                size: Size(size),
                align: Align(align),
            }
        }
        TyKind::Adt { def, .. } => adt_layout(db, *def),
        kind => Layout::of_primitive(kind),
    }
}

fn adt_layout(_db: &TypeDatabase, _def: DefId) -> Layout {
    Layout {
        size: Size(8),
        align: Align(8),
    }
}
