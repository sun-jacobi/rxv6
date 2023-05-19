use crate::arch::{MAXVA, NCPU, PGSIZE};
// Physical memory layout

// qemu -machine virt is set up like this,
// based on qemu's hw/riscv/virt.c:
//
// 00001000 -- boot ROM, provided by qemu
// 02000000 -- CLINT
// 0C000000 -- PLIC
// 10000000 -- uart0
// 10001000 -- virtio disk
// 80000000 -- boot ROM jumps here in machine mode
//             -kernel loads the kernel here
// unused RAM after 80000000.

// the kernel uses physical memory thus:
// 80000000 -- entry.S, then kernel text and data
// end -- start of kernel page allocation area
// PHYSTOP -- end RAM used by the kernel

// qemu puts UART registers here in physical memory.
pub const UART: u64 = 0x10000000;
pub const UART0_IRQ: u64 = 10;

// virtio mmio interface
pub const VIRTIO0: u64 = 0x10001000;
pub const VIRTIO0_IRQ: u64 = 1;

// core local interruptor (CLINT), which contains the timer.
pub const CLINT: u64 = 0x2000000;
pub const CLINT_MTIME: u64 = CLINT + 0xBFF8; // cycles since boot.

// qemu puts platform-level interrupt controller (PLIC) here.
pub const PLIC: u64 = 0x0c000000;

// the kernel expects there to be RAM
// for use by the kernel and user pages
// from physical address 0x80000000 to PHYSTOP.
pub const KERNBASE: u64 = 0x80000000;
pub const PHYSTOP: u64 = KERNBASE + 128 * 1024 * 1024;

// map the trampoline page to the highest address,
// in both user and kernel space.
pub const TRAMPOLINE: u64 = MAXVA - PGSIZE;

// map kernel stacks beneath the trampoline,
// each surrounded by invalid guard pages.
#[inline]
pub fn kstack(pid: usize) -> u64 {
    TRAMPOLINE - PGSIZE * 2 * (pid + 1) as u64
}

// a scratch area per CPU for machine-mode timer interrupts.
pub static mut SCRATCH: [[u64; 5]; NCPU] = [[0; 5]; NCPU];

// User memory layout.
// Address zero first:
//   text
//   original data and bss
//   fixed-size stack
//   expandable heap
//   ...
//   TRAPFRAME (p->trapframe, used by the trampoline)
//   TRAMPOLINE (the same page as in the kernel)
pub const _TRAPFRAME: u64 = TRAMPOLINE - PGSIZE;

extern "C" {
    static mut end: u64; // the first address after the kernel
    static mut etext: u64; // kernel code
    static mut trampoline: u64; // trap code position
    static mut timervec: u64; // machine-mode timer interrupt code
    static mut kernelvec: u64; // kernel trap code
}

pub unsafe fn init_linker_variable() {
    END = (&end as *const u64) as u64;
    ETEXT = (&etext as *const u64) as u64;
    TRAPTEXT = (&trampoline as *const u64) as u64;
    TIMERVEC = (&timervec as *const u64) as u64;
    KERNELVEC = (&kernelvec as *const u64) as u64;
}

pub static mut END: u64 = 0;
pub static mut ETEXT: u64 = 0;
pub static mut TRAPTEXT: u64 = 0;
pub static mut TIMERVEC: u64 = 0;
pub static mut KERNELVEC: u64 = 0;
