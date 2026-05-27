//! Stable newtype identifiers for compiler data structures.

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[repr(transparent)]
        pub struct $name(pub u32);

        impl $name {
            pub fn from_usize(index: usize) -> Self {
                Self(index as u32)
            }

            pub fn index(self) -> usize {
                self.0 as usize
            }
        }
    };
}

define_id!(LocalId);
define_id!(BlockId);
define_id!(SymbolId);
define_id!(DefId);
define_id!(TypeId);
define_id!(TraitId);
define_id!(ImplId);
define_id!(CrateId);
define_id!(ModuleId);
define_id!(ExprId);
define_id!(StmtId);
define_id!(ItemId);
define_id!(PatId);
define_id!(TyId);
define_id!(HIRBodyId);
define_id!(MirFuncId);
