pub(crate) mod buddy;
pub(crate) mod list;

use core::alloc::{GlobalAlloc, Layout};

use crate::lock::spinlock::SpinLock;

use self::buddy::BuddyAlloc;

use super::layout::HEAP_START;

pub(crate) const HEAP_SIZE: u64 = 64 * 1024 * 1024; // 64MiB

/// kernel heap memory alloctor
pub(crate) struct KernelAllocator {
    inner: SpinLock<BuddyAlloc<12, 4>>,
}

unsafe impl Sync for KernelAllocator {}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.lock().alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.lock().dealloc(ptr, layout)
    }
}

#[global_allocator]
static KERNEL_HEAP: KernelAllocator = KernelAllocator {
    inner: SpinLock::new(BuddyAlloc::new()),
};

pub(crate) fn init_kernel_heap() {
    unsafe {
        KERNEL_HEAP
            .inner
            .lock()
            .insert(HEAP_START, HEAP_START + HEAP_SIZE);
    }
}
