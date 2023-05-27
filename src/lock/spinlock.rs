use crate::arch::{cpu_id, intr_off, intr_on};
use crate::process::cpu::CMASTER;
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut, Drop};
use core::sync::atomic::Ordering::{Acquire, Release};
use core::{cell::UnsafeCell, sync::atomic::AtomicBool};
use riscv::register::sstatus;

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
        let old = sstatus::read().sie();
        intr_off(); // disable the interrupt to avoid the deadlock.
        let cpu = unsafe { CMASTER.my_cpu_mut() };
        let hart = cpu_id();
        cpu.intr = old;
        cpu.nlock += 1;
        while self.locked.swap(true, Acquire) {
            spin_loop();
        }
        Guard { lock: self, hart }
    }

    pub(crate) fn unlock(&self) {
        self.locked.store(false, Release);
        let cpu = unsafe { CMASTER.my_cpu_mut() };
        if cpu.nlock == 0 {
            return;
        }
        cpu.nlock -= 1;
        if cpu.nlock == 0 && cpu.intr {
            intr_on();
        }
    }
}

pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
    hart: usize,
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
        let cpu = unsafe { CMASTER.my_cpu_mut() };
        if cpu.nlock == 0 {
            return;
        }
        cpu.nlock -= 1;
        if cpu.nlock == 0 && cpu.intr {
            intr_on();
        }
    }
}
