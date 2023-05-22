use super::cpu::Context;
use crate::arch::{intr_off, intr_on, NPROC};
use crate::lock::spinlock::Guard;
use crate::lock::spinlock::SpinLock;
use crate::memory::layout::kstack;
use crate::process::cpu::CMASTER;
use crate::process::proc::{Proc, State};
use core::cell::OnceCell;
use core::ops::Index;

pub(crate) static mut PMASTER: PMaster = PMaster::new();

// Per-CPU process scheduler.
pub(crate) struct PMaster {
    procs: OnceCell<[SpinLock<Proc>; NPROC]>,
}

unsafe impl Sync for PMaster {}

impl PMaster {
    pub(crate) const fn new() -> Self {
        Self {
            procs: OnceCell::new(),
        }
    }

    pub(crate) fn init(&self) {
        let mut procs: [SpinLock<Proc>; NPROC] = [0; NPROC].map(|_| SpinLock::new(Proc::new()));
        for (pid, proc) in procs.iter_mut().enumerate() {
            if proc.lock().kstack.set(kstack(pid)).is_err() {
                panic!("reinitialized process table");
            }
        }
        if self.procs.set(procs).is_err() {
            panic!("failed to init the process table");
        }
    }

    // Return the current struct proc *, or zero if none.
    pub(crate) fn my_proc(&self) -> Option<Guard<Proc>> {
        intr_on();
        let cpu = unsafe { CMASTER.my_cpu() };
        intr_off();
        Some(self[cpu.pin?].lock())
    }

    // Each CPU calls scheduler() after setting itself up.
    // Scheduler never returns.  It loops, doing:
    //  - choose a process to run.
    //  - swtch to start running that process.
    //  - eventually that process transfers control
    //    via swtch back to the scheduler.
    pub(crate) fn scheduler(&mut self) -> ! {
        // it is safe because the interrupt is not on.
        let my_cpu = unsafe { CMASTER.my_cpu_mut() };
        loop {
            intr_on();
            for i in 0..NPROC {
                let mut proc = self[i].lock();
                if let State::Runnable = proc.state {
                    // context should contain something
                    assert!(!proc.context.is_null());
                    // cpu should not master any process now.
                    assert!(my_cpu.pin.is_none());
                    proc.state = State::Running;
                    my_cpu.pin = Some(i);
                    unsafe {
                        // Switch to chosen process.
                        swtch(my_cpu.context, proc.context);
                    }
                    my_cpu.pin = None; // cpu master no process now
                }
            }
        }
    }

    // Switch to scheduler.  Must hold only p->lock
    // and have changed proc->state. Saves and restores
    // intena because intena is a property of this
    // kernel thread, not this CPU. It should
    // be proc->intena and proc->noff, but that would
    // break in the few places where a lock is held but
    // there's no process.
    pub(crate) fn sched(&self, proc: Guard<Proc>) {
        if let State::Running = proc.state {
            panic!("process should not be running");
        }
        // switch to scheduler
        unsafe {
            swtch(proc.context, CMASTER.my_cpu().context);
        }
    }

    // Give up the CPU for one scheduling round.
    pub(crate) fn step(&self) {
        let mut proc = if let Some(proc) = self.my_proc() {
            proc
        } else {
            panic!("CPU should contain a process");
        };
        proc.state = State::Runnable;
        self.sched(proc);
    }
}

impl Index<usize> for PMaster {
    type Output = SpinLock<Proc>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.procs.get().unwrap()[index]
    }
}

extern "C" {
    // swtch.S
    fn swtch(old: *mut Context, new: *mut Context);
}
