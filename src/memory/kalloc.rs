use core::ptr::{read_volatile, write_volatile};

const PGSIZE: usize = 4096;

pub struct Kalloc {
    head: Option<*mut Page>, // pointer to head
}

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct Page {
    next: Option<*mut Page>,
}
impl Kalloc {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn insert(&mut self, start: u64, end: u64) {
        let mut page = start;
        while page < end {
            self.free(page);
            page += PGSIZE as u64;
        }
    }

    pub fn alloc(&mut self) -> Option<u64> {
        let ptr = if let Some(addr) = self.head {
            addr
        } else {
            return None;
        };
        let page = unsafe { read_volatile(ptr) };
        self.head = page.next;
        return Some(ptr as u64);
    }

    pub fn free(&mut self, addr: u64) {
        let ptr = addr as *mut Page;
        let head = Page { next: self.head };
        unsafe {
            write_volatile(ptr, head);
        }
        self.head = Some(ptr);
    }
}

