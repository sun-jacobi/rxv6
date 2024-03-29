use super::cpu::{Context, TrapFrame};
use crate::lock::spinlock::SpinLock;
use core::{cell::OnceCell, ptr};

/// State of a Process
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum State {
    Unused,
    Used,
    Sleeping,
    Runnable,
    Running,
    Zombie,
}

// Per-process state
pub(crate) struct Proc {
    pub(crate) info: SpinLock<ProcInfo>, // Process state
    pub(crate) context: ProcContext,     // Process context
}

pub(crate) struct ProcInfo {
    pub(crate) state: State,
    pub(crate) _hart_id: usize,
}

pub(crate) struct ProcContext {
    pub(crate) kstack: OnceCell<u64>, // Virtual address of kernel stack
    pub(crate) context: Context,      // swtch() here to run process
    pub(crate) pid: usize,
    pub(crate) trapframe: *mut TrapFrame,
    pub(crate) pagetable: u64,
}

impl ProcInfo {
    pub(crate) fn new() -> Self {
        Self {
            state: State::Unused,
            _hart_id: 42,
        }
    }
}

impl ProcContext {
    pub(crate) fn new() -> Self {
        Self {
            kstack: OnceCell::new(),
            context: Context::default(),
            pid: 0,
            pagetable: 0,
            trapframe: ptr::null_mut(),
        }
    }
}

impl Proc {
    pub(crate) fn new() -> Self {
        Self {
            info: SpinLock::new(ProcInfo::new()),
            context: ProcContext::new(),
        }
    }
}
