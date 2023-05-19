use crate::lock::spinlock::SpinLock;

// Physical memory allocator, for user processes,
// kernel stacks, page-table pages,
// and pipe buffers. Allocates whole 4096-byte pages.
use super::layout::{END, PHYSTOP};

const PGSIZE: usize = 4096;

pub(crate) static KALLOC: SpinLock<Kalloc> = SpinLock::new(Kalloc::new());

// Linked List Page Allocator
pub struct Kalloc {
    head: Option<*mut Page>,
}

unsafe impl Send for Kalloc {}

#[repr(C, align(4096))]
pub struct Page {
    next: Option<*mut Page>,
}

impl Kalloc {
    pub const fn new() -> Self {
        Self { head: None }
    }

    // init the kernel page allocator
    pub fn kinit() {
        unsafe {
            KALLOC.lock().insert(END, PHYSTOP);
        }
    }

    // free a allocated page
    pub fn insert(&mut self, start: u64, end: u64) {
        let mut page = start;
        while page < end {
            self.free(page);
            page += PGSIZE as u64;
        }
    }

    // allocate a new page
    // caller should be responsible for clearing the page
    pub fn alloc(&mut self) -> Option<u64> {
        let ptr = self.head?;
        let page = unsafe { ptr.as_mut().unwrap() };
        let _ = core::mem::replace(&mut self.head, page.next);
        Some(ptr as u64)
    }

    // append to free list
    pub fn free(&mut self, addr: u64) {
        let ptr = addr as *mut Page;
        let head = &mut self.head;
        let next = core::mem::replace(head, Some(ptr));
        unsafe {
            ptr.as_mut().unwrap().next = next;
        }
    }

    // mock test for page allocator
    pub fn _test() {
        static mut POOL: [u8; PGSIZE * 2] = [0; PGSIZE * 2];
        let start = unsafe { POOL.as_mut_ptr() as u64 };
        let end = start + (PGSIZE as u64) * 2;
        let mut kalloc = Kalloc::new();
        kalloc.insert(start, end);
        assert_eq!(kalloc.alloc().unwrap(), start + PGSIZE as u64);
        assert_eq!(kalloc.alloc().unwrap(), start);
        assert_eq!(kalloc.alloc(), None);
    }
}
