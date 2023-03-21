use crate::arch::{MAXVA, PGSIZE};

//pub const KERNBASE: u64 = 0x80000000;
//pub const PHYSTOP: u64 = KERNBASE + 128 * 1024 * 1024;
#[allow(dead_code)]
pub const TRAMPOLINE: u64 = MAXVA - PGSIZE;
#[allow(dead_code)]
pub const TRAPFRAME: u64 = TRAMPOLINE - PGSIZE;
