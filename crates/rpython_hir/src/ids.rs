macro_rules! define_hir_id {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
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

define_hir_id!(HirExprId);
define_hir_id!(HirStmtId);
define_hir_id!(HirPatId);
