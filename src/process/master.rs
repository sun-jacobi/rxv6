use super::cpu::{Context, CPU};
use crate::arch::{cpu_id, intr_off, intr_on, NCPU, NPROC};
use crate::lock::spinlock::Guard;
use crate::lock::spinlock::SpinLock;
use crate::memory::layout::kstack;
use crate::{println, print};
use crate::process::proc::{Proc, State};
use core::cell::OnceCell;
use core::ops::Index;


pub(crate) static mut MASTER: Master = Master::new();

// Per-CPU process scheduler.
pub(crate) struct Master {
    procs: OnceCell<[SpinLock<Proc>; NPROC]>,
    cpus: [CPU; NCPU],
}

unsafe impl Sync for Master {}

impl Master {
    pub(crate) const fn new() -> Self {
        Master {
            procs: OnceCell::new(),
            cpus: [CPU::new(); NCPU],
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

    // Return this CPU's cpu struct.
    // Interrupts must be disabled.
    pub(crate) fn my_cpu(&self) -> &CPU {
        let cpu_id = cpu_id();
        &self.cpus[cpu_id]
    }

    pub(crate) fn my_cpu_mut(&mut self) -> &mut CPU {
        let cpu_id = cpu_id();
        &mut self.cpus[cpu_id]
    }

    // Return the current struct proc *, or zero if none.
    pub(crate) fn my_proc(&self) -> Option<Guard<Proc>> {
        intr_on();
        let cpu = self.my_cpu();
        intr_off();
        Some(self[cpu.pin?].lock())
    }

    // Each CPU calls scheduler() after setting itself up.
    // Scheduler never returns.  It loops, doing:
    //  - choose a process to run.
    //  - swtch to start running that process.
    //  - eventually that process transfers control
    //    via swtch back to the scheduler.
    pub(crate) fn scheduler(&self) -> ! {
        // it is safe because the interrupt is not on.
        let my_cpu = self.my_cpu();
        loop {
            intr_on();
            for i in 0..NPROC {

                {
                    let proc = self[i].lock();
                }
                let proc = self[i].lock();
                if let State::Runnable = proc.state {
                    // Switch to chosen process. It
                    // is the process's job to release
                    // its lock and then reacquire it
                    // before jumping back to us.
                    assert!(!proc.context.is_null());
                    unsafe {
                        swtch(my_cpu.context, proc.context);
                    }
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
            swtch(proc.context, self.my_cpu().context);
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

impl Index<usize> for Master {
    type Output = SpinLock<Proc>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.procs.get().unwrap()[index]
    }
}

extern "C" {
    // swtch.S
    fn swtch(old: *mut Context, new: *mut Context);
}
