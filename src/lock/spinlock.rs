use crate::process::cpu::CMASTER;
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut, Drop};
use core::sync::atomic::Ordering::{Acquire, Release};
use core::{cell::UnsafeCell, sync::atomic::AtomicBool};

// Thanks to Mara Bos's brilliant book!
// https://marabos.nl/atomics/
pub(crate) struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub(crate) const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub(crate) fn lock(&self) -> Guard<T> {
        unsafe {
            CMASTER.push_off();
        }
        while self.locked.swap(true, Acquire) {
            spin_loop();
        }
        Guard { lock: self }
    }

    pub(crate) unsafe fn unlock(&self) {
        self.locked.store(false, Release);
        unsafe {
            CMASTER.pop_off();
        }
    }
}

pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release);
        unsafe {
            CMASTER.pop_off();
        }
    }
}
