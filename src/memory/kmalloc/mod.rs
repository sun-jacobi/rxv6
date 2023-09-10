pub(crate) mod buddy;
pub(crate) mod list;

use core::alloc::{GlobalAlloc, Layout};

use crate::lock::spinlock::SpinLock;

use self::buddy::BuddyAlloc;

use super::layout::HEAP_START;

pub(crate) const HEAP_SIZE: u64 = 64 * 1024 * 1024; // 64MiB

/// kernel heap memory alloctor
pub(crate) struct Kmalloc(SpinLock<BuddyAlloc<12, 4>>);

unsafe impl Sync for Kmalloc {}

unsafe impl GlobalAlloc for Kmalloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(ptr, layout)
    }
}

#[global_allocator]
static KMALLOC: Kmalloc = Kmalloc(SpinLock::new(BuddyAlloc::new()));

impl Kmalloc {
    pub(crate) fn init_kernel_heap() {
        unsafe {
            KMALLOC.0.lock().insert(HEAP_START, HEAP_START + HEAP_SIZE);
        }
    }
}
