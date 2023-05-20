use super::cpu::{Context, CPU};
use crate::arch::{cpu_id, NCPU, NPROC};
use crate::lock::spinlock::SpinLock;
use crate::memory::layout::kstack;
use crate::process::proc::{Proc, State};
use core::cell::OnceCell;
use core::ops::Index;

pub(crate) static MASTER: Master = Master::new();

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

    pub(crate) fn my_cpu(&self) -> &CPU {
        let cpu_id = cpu_id();
        &self.cpus[cpu_id]
    }

    // Each CPU calls scheduler() after setting itself up.
    // Scheduler never returns.  It loops, doing:
    //  - choose a process to run.
    //  - swtch to start running that process.
    //  - eventually that process transfers control
    //    via swtch back to the scheduler.
    pub(crate) fn scheduler(&self) -> ! {
        let my_cpu = self.my_cpu();
        loop {
            for i in 0..NPROC {
                let mut proc = self[i].lock();
                if let State::Runnable = proc.state {
                    // Switch to chosen process. It 
                    // is the process's job to release 
                    // its lock and then reacquire it
                    // before jumping back to us.
                    proc.state = State::Running;
                    assert!(!proc.context.is_null());
                    unsafe {
                        swtch(my_cpu.context, proc.context);
                    }
                    
                }
            }
        }
    }
}

impl Index<usize> for Master {
    type Output = SpinLock<Proc>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.procs.get().unwrap()[index]
    }
}

extern "C" {
    fn swtch(old: *const Context, new: *const Context);
}
