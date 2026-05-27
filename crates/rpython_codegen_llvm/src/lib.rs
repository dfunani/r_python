//! Native code generation (C backend by default; optional LLVM stub).

mod c;

pub use c::{compile_crate_to_c, COutput};

#[cfg(feature = "llvm")]
pub fn compile_llvm_stub() {
    // Optional LLVM backend — not required for default builds.
}
