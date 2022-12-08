// Physical memory allocator, for user processes,
// kernel stacks, page-table pages,
// and pipe buffers. Allocates whole 4096-byte pages.

use crate::memlayout::{KERNBASE, PHYSTOP};
use crate::risc::PG_SIZE;

use crate::address::Addr;

use crate::memlayout::end; // first address after kernel.
                           // defined by kernel.ld.

const FREE_LIST_SIZE: usize = (PHYSTOP - KERNBASE) / PG_SIZE;

pub(crate) struct Kallocator {
    free_list: [usize; FREE_LIST_SIZE],
    pivot: usize, // pivot for free_list
                  // spinlock (TODO)
}

impl Kallocator {
    pub(crate) fn init() -> Self {
        let mut kallocator = Self {
            free_list: [0; FREE_LIST_SIZE],
            pivot: 0,
        };
        kallocator.free_range(unsafe { end }.into(), PHYSTOP.into());
        kallocator
    }

    pub(crate) fn free_range(&mut self, pa_start: Addr, pa_end: Addr) {
        let mut start = pa_start.round_up_pg();
        while start + PG_SIZE <= pa_end {
            self.kfree(start);
            start = start + PG_SIZE;
        }
    }

    // Free the page of physical memory pointed at by pa,
    // which normally should have been returned by a
    // call to kalloc().  (The exception is when
    // initializing the allocator; see kinit above.)
    pub(crate) fn kfree(&mut self, pa: Addr) {
        // Fill with junk to catch dangling refs.
        Self::memset(pa, 1, PG_SIZE);
        self.push_back(pa.into());
    }

    // Allocate one 4096-byte page of physical memory.
    // Returns a pointer that the kernel can use.
    // Returns 0 if the memory cannot be allocated.
    pub(crate) fn kalloc(&mut self) -> Option<Addr> {
        self.pop_back()
    }

    pub(crate) fn memset(base_addr: Addr, val: u8, sz: usize) {
        for offset in 0..sz {
            let addr: usize = (base_addr + offset).into();
            let ptr = addr as *mut u8;
            unsafe {
                ptr.write(val);
            }
        }
    }

    // append addr to the free list
    pub(crate) fn push_back(&mut self, addr: usize) {
        self.free_list[self.pivot] = addr;
        self.pivot += 1;
    }

    // pop the last element in free list
    pub(crate) fn pop_back(&mut self) -> Option<Addr> {
        if self.pivot == 0 {
            return None;
        } else {
            self.pivot -= 1;
            return Some(self.free_list[self.pivot].into());
        }
    }
}
