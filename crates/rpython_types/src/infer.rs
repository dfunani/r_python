/// Inference variable index (fresh type variables during checking).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct InferVar(pub u32);

impl InferVar {
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    pub fn index(self) -> u32 {
        self.0
    }
}
