use super::cpu::{Context, TrapFrame};
use crate::arch::{intr_on, NPROC, PGSIZE};
use crate::layout::TRAPFRAME;
use crate::lock::spinlock::SpinLock;
use crate::memory::layout::{kstack_end, kstack_start, TRAMPOLINE, TRAPTEXT};
use crate::memory::vm::{PageTable, PTE_R, PTE_W, PTE_X};
use crate::process::cpu::CMASTER;
use crate::process::proc::{Proc, State};
use crate::trap::forkret;
use core::cell::OnceCell;
use core::ops::{Add, Index, IndexMut};
use core::ptr;

extern "C" {
    // swtch.S
    fn swtch(old: *mut Context, new: *mut Context);
}

pub(crate) static PID: SpinLock<usize> = SpinLock::new(0);

pub(crate) static mut PMASTER: PMaster = PMaster::new();

// Per-CPU process scheduler.
pub(crate) struct PMaster {
    procs: OnceCell<[Proc; NPROC]>,
}

unsafe impl Sync for PMaster {}

impl PMaster {
    pub(crate) const fn new() -> Self {
        Self {
            procs: OnceCell::new(),
        }
    }

    pub(crate) fn init(&self) {
        let mut procs: [Proc; NPROC] = [0; NPROC].map(|_| Proc::new());
        for (pid, proc) in procs.iter_mut().enumerate() {
            if proc.kstack.set(kstack_start(pid)).is_err() {
                panic!("failed to load process table");
            }
        }
        if self.procs.set(procs).is_err() {
            panic!("failed to load the process table");
        }
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
                let proc = &mut self[i];
                let mut proc_info = proc.info.lock();
                if let State::Runnable = proc_info.state {
                    // cpu should not master any process now.
                    assert!(my_cpu.pin.is_none());
                    proc_info.state = State::Running;
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
    pub(crate) fn sched(&mut self) {
        let cpu = unsafe { CMASTER.my_cpu_mut() };
        let proc = unsafe { &mut self[CMASTER.my_proc()] };
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
        let pin = unsafe { CMASTER.my_proc() };
        self.index_mut(pin).info.lock().state = State::Runnable;
        self.sched();
        self.index_mut(pin).info.unlock();
    }

    // Look in the process table for an UNUSED proc.
    // If found, initialize state required to run in the kernel,
    // and return with p->lock held.
    // If there are no free procs, or a memory allocation fails, return 0
    pub(crate) fn alloc(&mut self) -> Option<usize> {
        for pin in 0..NPROC {
            let proc = &mut self[pin];
            let mut proc_info = proc.info.lock();
            if let State::Unused = proc_info.state {
                let new_pid = Self::alloc_pid();
                proc.pid = new_pid;
                proc_info.state = State::Used;
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
                context.ra = forkret as u64;
                context.sp = kstack_end(pin);
                proc.context = context;
                return Some(pin);
            }
        }

        // no free procs
        None
    }

    fn alloc_pid() -> usize {
        let mut curr = PID.lock();
        *curr = curr.add(1);
        *curr
    }

    // Set up first user process.
    pub(crate) fn userinit(&mut self) {
        let proc = if let Some(pin) = self.alloc() {
            &mut self[pin]
        } else {
            panic!("failed to allocate the first process");
        };
        if let None = PageTable::uvmfirst(proc.pagetable) {
            panic!("failed to create page table for the first procress");
        }

        let trapframe = proc.trapframe;
        unsafe {
            (*trapframe).epc = 0; // user program counter
            (*trapframe).sp = PGSIZE; // user stack pointer
        }
        proc.info.lock().state = State::Runnable;
    }
}

impl Index<usize> for PMaster {
    type Output = Proc;
    fn index(&self, index: usize) -> &Self::Output {
        &self.procs.get().unwrap()[index]
    }
}

impl IndexMut<usize> for PMaster {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.procs.get_mut().unwrap().index_mut(index)
    }
}

pub(crate) const INITCODE: [u8; 12] = [
    0x13, 0x05, 0x10, 0x00, 0x93, 0x05, 0x20, 0x00, 0x6f, 0xf0, 0x9f, 0xff,
];
