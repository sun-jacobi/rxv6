use core::ptr;

// Saved registers for kernel context switches.
#[repr(C)]
pub(crate) struct Context {
    ra: u64,
    sp: u64,

    // callee-saved register
    s0: u64,
    s1: u64,
    s2: u64,
    s3: u64,
    s4: u64,
    s5: u64,
    s6: u64,
    s7: u64,
    s8: u64,
    s9: u64,
    s10: u64,
    s11: u64,
}

// Per-CPU state.
pub(crate) struct CPU {
    pub(crate) context: *const Context,
}

impl CPU {
    pub(crate) const fn new() -> Self {
        CPU { context:  ptr::null()}
    }
}
