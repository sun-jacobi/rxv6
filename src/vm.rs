use crate::memlayout::Addr;

type PTE = usize;
type PageTable = Addr;
use crate::kalloc::Kallocator;
use crate::risc::PG_SIZE;

pub(crate) struct KVM {
    kallocator: Kallocator, // the kernel's memory allocator
    page_table: Addr,       // the kernel's page table.
}

impl KVM {
    // Initialize the one kernel_pagetable
   
    pub(crate) fn new() -> Self {
        let mut kallocator = Kallocator::init();
        let page_table = kallocator.kalloc();
        Kallocator::memset(page_table, 0, PG_SIZE);
        Self {
            kallocator,
            page_table,
        }
    }


    // add a mapping to the kernel page table.
    // only used when booting.
    // does not flush TLB or enable paging.

    // Create PTEs for virtual addresses starting at va that refer to
    // physical addresses starting at pa. va and size might not
    // be page-aligned. Returns 0 on success, -1 if walk() couldn't
    // allocate a needed page-table page.
    pub(crate) fn kvmmap(self, virt_addr: Addr, phys_addr: Addr, sz: usize, perm: u8) {
        let mut down = virt_addr.round_down_pg();
        let up = (virt_addr + sz - 1).round_down_pg();
        let mut target = phys_addr;
        while down < up {
            let pte = self.walk(virt_addr.into(), true);
            unsafe {
                (pte as *mut usize).write(phys_addr.into());
            }
            down = down + PG_SIZE;
            target = target + PG_SIZE;
        }
    }

    // Return the address of the PTE in page table pagetable
    // that corresponds to virtual address va.  If alloc!=0,
    // create any required page-table pages.
    //
    // The risc-v Sv39 scheme has three levels of page-table
    // pages. A page-table page contains 512 64-bit PTEs.
    // A 64-bit virtual address is split into five fields:
    //   39..63 -- must be zero.
    //   30..38 -- 9 bits of level-2 index.
    //   21..29 -- 9 bits of level-1 index.
    //   12..20 -- 9 bits of level-0 index.
    //    0..11 -- 12 bits of byte offset within the page.
    fn walk(&self, virt_addr: usize, _alloc: bool) -> PTE {
        let mut ppn: usize = self.page_table.into();
        let mut pte;
        for level in 2..=0 {
            pte = ppn + ((virt_addr >> 12) >> (9 * level)) * 8;
            ppn = (unsafe { (pte as *const usize).read() } >> 10) << 12;
        }
        ppn | (virt_addr | 0xfff)
    }
}
