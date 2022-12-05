// A Simple Handmade mutex
pub(crate) struct SpinLock {
    locked: bool,
}

impl SpinLock {
    pub(crate) fn new() -> Self {
        Self { locked: false }
    }

    // TODO
    // Acquire the lock.
    // Loops (spins) until the lock is acquired.
    pub(crate) fn lock(&self) {
        // disable interrupts to avoid deadlock.

        // On RISC-V, sync_lock_test_and_set turns into an atomic swap:
        //   a5 = 1
        //   s1 = &lk->locked
        //   amoswap.w.aq a5, a5, (s1)
    }

    // Release the lock.
    pub(crate) fn unlock(&self) {}
}
