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
pub(crate) const UART0: Addr = Addr(0x10000000);
pub(crate) const UART0_IRQ: usize = 10;

// virtio mmio interface
pub(crate) const VIRTIO0: Addr = Addr(0x10001000);
pub(crate) const VIRTIO0_IRQ: usize = 1;

// core local interruptor (CLINT), which contains the timer.
pub(crate) const CLINT: usize = 0x2000000;
#[allow(non_snake_case)]
pub(crate) fn CLINT_MTIMECMP(hart_id: usize) -> usize {
    CLINT + 0x4000 + 8 * (hart_id)
}

pub(crate) const CLINT_MTIME: usize = CLINT + 0xBFF8; // cycles since boot.

// qemu puts platform-level interrupt controller (PLIC) here.
pub(crate) const PLIC: Addr = Addr(0x0c000000);

// the kernel expects there to be RAM
// for use by the kernel and user pages
// from physical address 0x80000000 to PHYSTOP.
pub(crate) const KERNBASE: usize = 0x80000000;
pub(crate) const PHYSTOP: usize = KERNBASE + 128 * 1024 * 1024;

// map the trampoline page to the highest address,
// in both user and kernel space.
use crate::risc::{MAXVA, PG_SIZE};

pub(crate) const TRAMPOLINE: usize = MAXVA - PG_SIZE;

// map kernel stacks beneath the trampoline,
// each surrounded by invalid guard pages.
#[allow(non_snake_case)]
pub(crate) fn KSTACK(p: usize) -> usize {
    TRAMPOLINE - (p + 1) * 2 * PG_SIZE
}

// User memory layout.
// Address zero first:
//   text
//   original data and bss
//   fixed-size stack
//   expandable heap
//   ...
//   TRAPFRAME (p->trapframe, used by the trampoline)
//   TRAMPOLINE (the same page as in the kernel)

pub(crate) const TRAPFRAME: usize = TRAMPOLINE - PG_SIZE;

extern "C" {
    pub(crate) static etext: usize; // first address after kernel.
    pub(crate) static end: usize; // first address after code section.
    pub(crate) static trampoline: usize;
}

use crate::address::Addr;
