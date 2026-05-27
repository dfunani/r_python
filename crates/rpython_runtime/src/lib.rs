//! rPython runtime — C ABI static library linked into every binary.

#![allow(dead_code)]

/// Path to the runtime C source (for `cc` crate in the driver).
pub fn runtime_c_src() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/src/rt.c")
}

/// Exported C ABI symbol names.
pub mod symbols {
    pub const RT_INIT: &str = "rpy_rt_init";
    pub const RT_FINI: &str = "rpy_rt_fini";
    pub const PANIC: &str = "rpy_panic";
    pub const PRINT_STR: &str = "rpy_print_str";
    pub const PRINT_INT: &str = "rpy_print_int";
    pub const PRINT_BOOL: &str = "rpy_print_bool";
    pub const PRINT_NEWLINE: &str = "rpy_print_newline";
    pub const STR_EQ: &str = "rpy_str_eq";
    pub const ALLOC: &str = "rpy_alloc";
    pub const FREE: &str = "rpy_free";
}
