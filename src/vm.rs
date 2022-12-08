use crate::address::Addr;

type PTE = usize;
use crate::kalloc::Kallocator;
use crate::memlayout::{etext, KERNBASE, PHYSTOP};
use crate::memlayout::{trampoline, TRAMPOLINE};
use crate::memlayout::{PLIC, UART0, VIRTIO0};
use crate::risc::PG_SIZE;
use crate::risc::PTE_V;
use crate::risc::{PTE_R, PTE_W, PTE_X};

pub(crate) struct KVM {
    kallocator: Kallocator, // the kernel's memory allocator
    page_table: Addr,       // the kernel's page table.
}

impl KVM {
    // Initialize the one kernel_pagetable
    pub(crate) fn new() -> Self {
        let mut kallocator = Kallocator::init();
        let page_table = kallocator.kalloc().unwrap();
        let kvm = Self {
            kallocator,
            page_table,
        };
        kvm.make().hart()
    }

    // Switch h/w page table register to the kernel's page table,
    // and enable paging.
    fn hart(self) -> Self {
        use riscv::register;
        unsafe {
            riscv::asm::sfence_vma_all();
        }
        register::satp::write(self.page_table.into()); // enable page table
        unsafe {
            riscv::asm::sfence_vma_all();
        }
        self
    }

    // Make a direct-map page table for the kernel.
    fn make(mut self) -> Self {
        #[allow(non_snake_case)]
        let ETXT = unsafe { etext };
        // uart registers
        self.kvmmap(UART0, UART0, PG_SIZE, PTE_R | PTE_W);
        // virtio mmio disk interface
        self.kvmmap(VIRTIO0, VIRTIO0, PG_SIZE, PTE_R | PTE_W);
        // PLIC
        self.kvmmap(PLIC, PLIC, 0x400000, PTE_R | PTE_W);
        // map kernel text executable and read-only.

        self.kvmmap(
            Addr::from(KERNBASE),
            Addr::from(KERNBASE),
            ETXT - KERNBASE,
            PTE_R | PTE_X,
        );

        // map kernel data and the physical RAM we'll make use of.
        self.kvmmap(
            Addr::from(ETXT),
            Addr::from(ETXT),
            PHYSTOP - ETXT,
            PTE_R | PTE_W,
        );

        // map the trampoline for trap entry/exit to
        // the highest virtual address in the kernel.
        self.kvmmap(
            Addr(TRAMPOLINE),
            Addr(unsafe { trampoline }),
            PG_SIZE,
            PTE_R | PTE_X,
        );

        self
    }

    // add a mapping to the kernel page table.
    // only used when booting.
    // does not flush TLB or enable paging.

    // Create PTEs for virtual addresses starting at va that refer to
    // physical addresses starting at pa. va and size might not
    // be page-aligned. Returns 0 on success, -1 if walk() couldn't
    // allocate a needed page-table page.
    fn kvmmap(&mut self, virt_addr: Addr, phys_addr: Addr, sz: usize, perm: usize) {
        let mut down = virt_addr.round_down_pg();
        let up = (virt_addr + sz - 1).round_down_pg();
        let mut target = phys_addr;
        while down < up {
            let pte = self.walk(virt_addr.into(), true);
            match pte {
                None => panic!("vm : failed to create a pte."),
                Some(addr) => unsafe {
                    let ptr = addr as *mut usize;
                    if (ptr.read() & (PTE_V as usize)) != 0 {
                        panic!("vm : remap on same pte.");
                    }
                    (addr as *mut usize).write(((usize::from(phys_addr)) << 10) | perm)
                },
            }
            down = down + PG_SIZE;
            target = target + PG_SIZE;
        }
    }

    // Return the address of the PTE in page table pagetable
    // that corresponds to virtual address va.
    // If alloc!=0,
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
    fn walk(&mut self, virt_addr: usize, alloc: bool) -> Option<PTE> {
        let mut ppn: usize = self.page_table.into();
        let mut pte_t;
        for level in 2..=0 {
            pte_t = ppn + Self::get_pg(virt_addr.into(), level) * 8;
            let pte = unsafe { (pte_t as *mut usize).read() };
            if (pte & PTE_V) == 0 {
                // already made
                ppn = (pte_t >> 10) << 12;
            } else if alloc {
                match self.kallocator.kalloc() {
                    Some(pg) => {
                        ppn = pg.into();
                        unsafe {
                            (pte_t as *mut usize).write(((ppn >> 12) << 10) | PTE_V);
                        }
                    }
                    None => return None,
                }
            } else {
                return None;
            }
        }
        Some(ppn | (virt_addr | 0xfff))
    }

    fn get_pg(virt_addr: usize, level: u8) -> usize {
        let tree = virt_addr >> 12;
        (tree >> (level * 9)) & (0x1FF) // only use 9 bits
    }
}
