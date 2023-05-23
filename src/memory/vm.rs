use core::{panic, slice};

use super::layout::{ETEXT, KERNBASE, PHYSTOP, UART, VIRTIO0};
use crate::arch::{NPROC, PGSIZE};
use crate::memory::kalloc::KALLOC;
use crate::memory::layout::{kstack_start, PLIC, TRAMPOLINE, TRAPTEXT};
use riscv::asm::sfence_vma_all;
use riscv::register::satp;
use riscv::register::satp::Mode;

// Virtual address : 39 = 9 + 9 + 9 + 12
// rxv6 use a 3 level Page table
// 9 bits represent the idx of each level
// 12 bits represent the offset in physical address
pub struct Kvm {
    root: u64,
}

// for each entry, [53:10] is PPN
// for level 1 and 2, PPN represent the physical address
// of next level table
// for level 3, PPN is the pa[53:10]
// level 3 PPN + Offset = pa
#[repr(C, align(4096))]
pub struct PageTable {
    ptes: &'static mut [u64],
}

const PPN_MASK: u64 = 0xFFFFFFFFFFF; // 44 bit
const OFFSET_MASK: u64 = 0xFFF; // [11:0]
pub const PTE_V: u64 = 1 << 0; // valid
pub const PTE_R: u64 = 1 << 1; // readable
pub const PTE_W: u64 = 1 << 2; // writable
pub const PTE_X: u64 = 1 << 3; // executable
pub const _PTE_U: u64 = 1 << 4; // user can access

// The risc-v Sv39 scheme has three levels of page-table
// pages. A page-table page contains 512 64-bit PTEs.
// A 64-bit virtual address is split into five fields:
//   39..63 -- must be zero.
//   30..38 -- 9 bits of level-2 index.
//   21..29 -- 9 bits of level-1 index.
//   12..20 -- 9 bits of level-0 index.
//    0..11 -- 12 bits of byte offset within the page.
pub enum PageTableLevel {
    Lv1,
    Lv2,
    Lv3,
}

impl Kvm {
    // create the kernel map
    pub fn init() -> Self {
        let mut kvm = PageTable::create_table();
        unsafe {
            // uart register
            kvm.map(UART, UART, PGSIZE, PTE_R | PTE_W);
            assert_eq!(kvm.translate(UART), UART);

            // PLIC
            kvm.map(PLIC, PLIC, 0x400000, PTE_R | PTE_W);
            assert_eq!(kvm.translate(PLIC), PLIC);

            // virtio mmio disk interface
            kvm.map(VIRTIO0, VIRTIO0, PGSIZE, PTE_R | PTE_W);
            assert_eq!(kvm.translate(VIRTIO0), VIRTIO0);

            // map kernel text excutable and read-only
            kvm.map(KERNBASE, KERNBASE, ETEXT - KERNBASE, PTE_R | PTE_X);

            assert_eq!(kvm.translate(KERNBASE), KERNBASE);

            // map kernel data and the physical RAM we'll make use of.
            kvm.map(ETEXT, ETEXT, PHYSTOP - ETEXT, PTE_R | PTE_W);
            assert_eq!(kvm.translate(ETEXT), ETEXT);

            // map the trampoline for trap entry/exit to
            // the highest virtual address in the kernel.
            kvm.map(TRAMPOLINE, TRAPTEXT, PGSIZE, PTE_R | PTE_X);
            assert_eq!(kvm.translate(TRAMPOLINE), TRAPTEXT);

            // allocate and map a kernel stack for each process.
            for p in 0..NPROC {
                let phys_addr = if let Some(addr) = KALLOC.lock().alloc() {
                    addr
                } else {
                    panic!("failed to create the kernel stack");
                };
                let virt_addr = kstack_start(p);
                kvm.map(virt_addr, phys_addr, PGSIZE, PTE_R | PTE_W);
                assert_eq!(kvm.translate(virt_addr), phys_addr);
            }
        }
        Self {
            root: kvm.base_addr(),
        }
    }

    // turn on the mmu hardware
    pub fn init_hart(&self) {
        let ppn = { (self.root >> 12) as usize };
        unsafe {
            // wait for any previous writes to the page table memory to finish.
            sfence_vma_all();
            satp::set(Mode::Sv39, 0, ppn);
            // flush stale entries from the TLB.
            sfence_vma_all();
        }
    }
}

impl PageTable {
    // map[virt_addr..virt_addr + range]
    // -> [phys_addr..phys_addr + range]
    pub fn map(&mut self, virt_addr: u64, phys_addr: u64, range: u64, perm: u64) {
        assert_eq!(range & (4096 - 1), 0); // range must be 4096-aligned
        let mut phys = phys_addr;
        let mut virt = virt_addr;
        let end = phys_addr + range;
        while phys < end {
            self.map_page(virt, phys, perm);
            phys += PGSIZE;
            virt += PGSIZE;
        }
    }

    pub fn map_page(&mut self, virt_addr: u64, phys_addr: u64, perm: u64) {
        // level 1
        let lv1_ptes = &mut self.ptes;
        let lv1_idx = Self::idx(virt_addr, PageTableLevel::Lv1);
        let lv1_pte = lv1_ptes[lv1_idx];
        let mut lv2_tbl = if Self::used(lv1_pte) {
            PageTable::from_pte(lv1_pte)
        } else {
            // allocate a new page for table
            PageTable::create_table()
        };
        lv1_ptes[lv1_idx] = lv2_tbl.to_pte() | PTE_V;

        //-----------------------------------------------------
        // level 2
        let lv2_ptes = &mut lv2_tbl.ptes;
        let lv2_idx = Self::idx(virt_addr, PageTableLevel::Lv2);
        let lv2_pte = lv2_ptes[lv2_idx];
        let mut lv3_tbl = if Self::used(lv2_pte) {
            PageTable::from_pte(lv2_pte)
        } else {
            PageTable::create_table()
        };
        lv2_tbl.ptes[lv2_idx] = lv3_tbl.to_pte() | PTE_V;

        //-----------------------------------------------------
        // level 3
        let lv3_ptes = &mut lv3_tbl.ptes;
        let lv3_idx = Self::idx(virt_addr, PageTableLevel::Lv3);
        let mut lv3_pte = lv3_ptes[lv3_idx];
        if Self::used(lv3_pte) {
            panic!("Virtual memory: remap fault");
        }
        lv3_pte = Self::ppn(phys_addr) << 10 | perm | PTE_V;
        lv3_tbl.ptes[lv3_idx] = lv3_pte;
    }

    // translate virtual address to physical address
    pub fn translate(&self, virt_addr: u64) -> u64 {
        // lv1
        let lv1_ptes = &self.ptes;
        let lv1_idx = Self::idx(virt_addr, PageTableLevel::Lv1);
        let lv1_pte = lv1_ptes[lv1_idx];
        let lv2_tbl = if Self::used(lv1_pte) {
            PageTable::from_pte(lv1_pte)
        } else {
            panic!("Virtual memory: invalid virtual address");
        };
        //------------------------------------
        // lv2
        let lv2_ptes = &lv2_tbl.ptes;
        let lv2_idx = Self::idx(virt_addr, PageTableLevel::Lv2);
        let lv2_pte = lv2_ptes[lv2_idx];
        let lv3_tbl = if Self::used(lv2_pte) {
            PageTable::from_pte(lv2_pte)
        } else {
            panic!("Virtual memory: invalid virtual address");
        };
        //-----------------------------------------------------
        // level 3
        let lv3_ptes = &lv3_tbl.ptes;
        let lv3_idx = Self::idx(virt_addr, PageTableLevel::Lv3);
        let lv3_pte = lv3_ptes[lv3_idx];
        if !Self::used(lv3_pte) {
            panic!("Virtual memory: invalid virtual address");
        }
        ((lv3_pte >> 10) << 12) | Self::offset(virt_addr)
    }

    pub fn base_addr(&mut self) -> u64 {
        self.ptes.as_ptr() as u64
    }

    pub fn from_addr(addr: u64) -> Self {
        let ptes = unsafe { slice::from_raw_parts_mut(addr as *mut u64, 512) };
        Self { ptes }
    }

    pub fn from_pte(pte: u64) -> Self {
        let ppn = (pte >> 10) << 12;
        Self::from_addr(ppn)
    }

    pub fn to_pte(&self) -> u64 {
        let addr = self.ptes.as_ptr() as u64;
        (addr >> 12) << 10
    }

    pub fn create_table() -> Self {
        let addr = if let Some(page) = KALLOC.lock().alloc() {
            page
        } else {
            panic!("failed to create page table");
        };
        let ptes = unsafe { slice::from_raw_parts_mut(addr as *mut u64, 512) };
        for pte in ptes.iter_mut() {
            *pte = 0;
        }
        Self { ptes }
    }

    // pagetable utilities
    fn offset(addr: u64) -> u64 {
        addr & OFFSET_MASK
    }

    fn idx(addr: u64, level: PageTableLevel) -> usize {
        let idx = match level {
            PageTableLevel::Lv1 => (addr >> (9 + 9 + 12)) & 511,
            PageTableLevel::Lv2 => (addr >> (9 + 12)) & 511,
            PageTableLevel::Lv3 => (addr >> (12)) & 511,
        };
        idx as usize
    }

    fn ppn(addr: u64) -> u64 {
        (addr >> 12) & PPN_MASK
    }

    fn used(pte: u64) -> bool {
        pte & PTE_V != 0
    }
}
