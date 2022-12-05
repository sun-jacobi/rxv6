#[allow(dead_code)]
type PageTable = *mut usize;

type PTE = *mut usize;

//use crate::kalloc::Kallocator;
//use crate::risc;

/*
pub(crate) fn kvm_init() -> PageTable {
    let kpgtbl = kalloc();
    memset(kpgtbl, 0, risc::PG_SIZE);

    kpgtbl
}*/

//pub(crate) fn walk(page_tbl: PageTable, virt_addr: usize) -> PTE {}

pub(crate) fn kvmmap(
    page_tbl: PageTable,
    virt_addr: usize,
    phys_addr: usize,
    sz: usize,
    perm: usize,
) {
}
