// Physical memory allocator, for user processes,
// kernel stacks, page-table pages,
// and pipe buffers. Allocates whole 4096-byte pages.
use super::layout::{END, PHYSTOP};
use core::ptr::{read_volatile, write_volatile};

const PGSIZE: usize = 4096;

pub struct Kalloc {
    head: Option<*mut Page>,
}

pub static mut KALLOC: Kalloc = Kalloc::new();

#[repr(C, align(4096))]
pub struct Page {
    next: Option<*mut Page>,
}

impl Kalloc {
    pub const fn new() -> Self {
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

    pub fn init() {
        unsafe {
            KALLOC.insert(END, PHYSTOP);
        }
    }
}
