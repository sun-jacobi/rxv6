use core::{panic, slice};

use super::kalloc::KALLOC;

// Virtual address : 39 = 9 + 9 + 9 + 12
// rxv6 use a 3 level Page table
// 9 bits represent the idx of each level
// 12 bits represent the offset in physical address
pub struct Kvm {
    pagetbl: PageTable,
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

pub enum PageTableLevel {
    LV1,
    LV2,
    LV3,
}

const PPN_MASK: u64 = 0xFFFFFFFFFFF; // 44 bit
const OFFSET_MASK: u64 = 0xFFF; // [11:0]
const PTE_V: u64 = 1 << 0; // valid
const PTE_R: u64 = 1 << 1; // readable
const PTE_W: u64 = 1 << 2; // writable
const PTE_X: u64 = 1 << 3; // executable
const PTE_U: u64 = 1 << 4; // user can access

impl Kvm {
    pub fn map(&mut self, phys_addr: u64, virt_addr: u64, perm: u64) {
        let lv1_tbl = &mut self.pagetbl;
        let lv1_idx = idx(virt_addr, PageTableLevel::LV1);
        let lv1_pte = lv1_tbl.ptes[lv1_idx];
        let lv2_tbl_pa = if used(lv1_pte) {
            ppn(lv1_pte)
        } else {
            if let Some(page) = unsafe { KALLOC.alloc() } {
                page
            } else {
                panic!("failed to allocate page");
            }
        };
        lv1_tbl.ptes[lv1_idx] = pte(lv2_tbl_pa) | perm | PTE_V;
        //-----------------------------------------------------
        let lv2_tbl = PageTable::from_addr(lv2_tbl_pa);
        let lv2_idx = idx(virt_addr, PageTableLevel::LV2);
        let lv2_pte = lv2_tbl.ptes[lv2_idx];
        let lv3_tbl_pa = if used(lv2_pte) {
            ppn(lv2_pte)
        } else {
            if let Some(page) = unsafe { KALLOC.alloc() } {
                page
            } else {
                panic!("Kvm: failed to allocate page");
            }
        };
        lv2_tbl.ptes[lv2_idx] = pte(lv3_tbl_pa) | perm | PTE_V;
        //-----------------------------------------------------
        let lv3_tbl = PageTable::from_addr(lv3_tbl_pa);
        let lv3_idx = idx(virt_addr, PageTableLevel::LV3);
        let mut lv3_pte = lv3_tbl.ptes[lv3_idx];
        if used(lv3_pte) {
            panic!("Kvm: remap fault");
        }
        lv3_pte = ppn(phys_addr) | offset(virt_addr) | perm | PTE_V;
        lv3_tbl.ptes[lv3_idx] = lv3_pte;
    }
}

impl PageTable {
    pub fn from_addr(addr: u64) -> Self {
        let ptes = unsafe { slice::from_raw_parts_mut(addr as *mut u64, 512) };
        Self { ptes }
    }
}

//===============================================
// utilities

#[inline]
fn idx(addr: u64, level: PageTableLevel) -> usize {
    let idx = match level {
        PageTableLevel::LV1 => addr >> (9 + 9 + 12),
        PageTableLevel::LV2 => addr >> (9 + 12),
        PageTableLevel::LV3 => addr >> (12),
    };
    idx as usize
}

#[inline]
fn offset(addr: u64) -> u64 {
    addr & OFFSET_MASK
}

#[inline]
fn ppn(addr: u64) -> u64 {
    (addr >> 10) & PPN_MASK
}

#[inline]
fn pte(phys_addr: u64) -> u64 {
    phys_addr << 10
}

#[inline]
fn used(pte: u64) -> bool {
    pte & PTE_V != 0
}
