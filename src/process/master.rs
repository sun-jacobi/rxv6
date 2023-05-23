use super::cpu::{Context, TrapFrame};
use crate::arch::{intr_off, intr_on, NPROC, PGSIZE};
use crate::layout::TRAPFRAME;
use crate::lock::spinlock::Guard;
use crate::lock::spinlock::SpinLock;
use crate::memory::layout::{kstack_end, kstack_start, TRAMPOLINE, TRAPTEXT};
use crate::memory::vm::{PageTable, PTE_R, PTE_W, PTE_X};
use crate::process::cpu::CMASTER;
use crate::process::proc::{Proc, State};
use crate::trap::usertrapret;
use core::cell::OnceCell;
use core::ops::{Add, Index};
use core::ptr;

extern "C" {
    // swtch.S
    fn swtch(old: *mut Context, new: *mut Context);
}

// a user program that calls exec("/init")
// assembled from ../user/initcode.S
// od -t xC ../user/initcode
pub(crate) const _INITCODE: [u8; 52] = [
    0x17, 0x05, 0x00, 0x00, 0x13, 0x05, 0x45, 0x02, 0x97, 0x05, 0x00, 0x00, 0x93, 0x85, 0x35, 0x02,
    0x93, 0x08, 0x70, 0x00, 0x73, 0x00, 0x00, 0x00, 0x93, 0x08, 0x20, 0x00, 0x73, 0x00, 0x00, 0x00,
    0xef, 0xf0, 0x9f, 0xff, 0x2f, 0x69, 0x6e, 0x69, 0x74, 0x00, 0x00, 0x24, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

pub(crate) static mut PMASTER: PMaster = PMaster::new();

// Per-CPU process scheduler.
pub(crate) struct PMaster {
    procs: OnceCell<[SpinLock<Proc>; NPROC]>,
    pid: SpinLock<usize>,
}

unsafe impl Sync for PMaster {}

impl PMaster {
    pub(crate) const fn new() -> Self {
        Self {
            procs: OnceCell::new(),
            pid: SpinLock::new(0),
        }
    }

    pub(crate) fn init(&self) {
        let mut procs: [SpinLock<Proc>; NPROC] = [0; NPROC].map(|_| SpinLock::new(Proc::new()));
        for (pid, proc) in procs.iter_mut().enumerate() {
            if proc.lock().kstack.set(kstack_start(pid)).is_err() {
                panic!("failed to load process table");
            }
        }
        if self.procs.set(procs).is_err() {
            panic!("failed to load the process table");
        }
    }

    // Return the current struct proc *, or zero if none.
    pub(crate) fn my_proc(&self) -> Option<Guard<Proc>> {
        intr_off();
        let cpu = unsafe { CMASTER.my_cpu() };
        let proc = self[cpu.pin?].lock();
        intr_on();
        Some(proc)
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
                    // cpu should not master any process now.
                    assert!(my_cpu.pin.is_none());
                    proc.state = State::Running;
                    my_cpu.pin = Some(i);
                    let old = ptr::addr_of_mut!(my_cpu.context);
                    let new = ptr::addr_of_mut!(proc.context);
                    // Switch to chosen process.
                    unsafe {
                        swtch(old, new);
                    }
                    my_cpu.pin = None; // cpu master no process now
                }
            } // proc unlock
        }
    }

    // Switch to scheduler.  Must hold only p->lock
    // and have changed proc->state. Saves and restores
    // intena because intena is a property of this
    // kernel thread, not this CPU. It should
    // be proc->intena and proc->noff, but that would
    // break in the few places where a lock is held but
    // there's no process.
    pub(crate) fn sched(&self, mut proc: Guard<Proc>) {
        if let State::Running = proc.state {
            panic!("process should not be running");
        }

        let cpu = unsafe { CMASTER.my_cpu_mut() };
        // proc.context = Context::default();
        let old = ptr::addr_of_mut!(proc.context);
        let new = ptr::addr_of_mut!(cpu.context);

        // switch to scheduler
        unsafe {
            swtch(old, new);
        }
    }

    // Give up the CPU for one scheduling round.
    pub(crate) fn step(&mut self) {
        let mut proc = if let Some(proc) = self.my_proc() {
            proc
        } else {
            panic!("CPU should contain a process");
        };
        proc.state = State::Runnable;
        self.sched(proc);
    }

    // Look in the process table for an UNUSED proc.
    // If found, initialize state required to run in the kernel,
    // and return with p->lock held.
    // If there are no free procs, or a memory allocation fails, return 0
    pub(crate) fn alloc(&mut self) -> Option<Guard<Proc>> {
        for pin in 0..NPROC {
            let mut proc = self[pin].lock();
            if let State::Unused = proc.state {
                proc.pid = self.alloc_pid();
                proc.state = State::Used;
                // Allocate a trapframe page.
                proc.trapframe = TrapFrame::new()?;
                // An empty user page tab
                let mut pagetable = PageTable::create_table();
                unsafe {
                    pagetable.map(TRAMPOLINE, TRAPTEXT, PGSIZE, PTE_R | PTE_X);
                    pagetable.map(TRAPFRAME, proc.trapframe as u64, PGSIZE, PTE_R | PTE_W);
                }

                proc.pagetable = pagetable.base_addr();

                // Set up new context to start executing at forkret,
                // which returns to user space.
                let mut context = Context::default();
                context.ra = usertrapret as u64;
                context.sp = kstack_end(pin);
                proc.context = context;

                return Some(proc);
            }
        }

        // no free procs
        None
    }

    pub(crate) fn alloc_pid(&self) -> usize {
        let mut curr = self.pid.lock();
        *curr = curr.add(1);
        *curr
    }

    // Set up first user process.
    pub(crate) fn userinit(&mut self) {
        let mut proc = if let Some(p) = self.alloc() {
            p
        } else {
            panic!("failed to create the first process");
        };
        // proc.trapframe.sp = PGSIZE; // user stack pointer
        // proc.trapframe.epc = 0; // user program counter
        proc.state = State::Runnable;
    }
}

impl Index<usize> for PMaster {
    type Output = SpinLock<Proc>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.procs.get().unwrap()[index]
    }
}
