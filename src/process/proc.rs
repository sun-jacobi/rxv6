use core::cell::OnceCell;

use super::cpu::Context;

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
    pub(crate) state: State,          // Process state
    pub(crate) kstack: OnceCell<u64>, // Virtual address of kernel stack
    pub(crate) context: Context,      // swtch() here to run process
    pub(crate) pid: usize,
}

impl Proc {
    pub(crate) fn new() -> Self {
        Self {
            state: State::Unused,
            kstack: OnceCell::new(),
            context: Context::default(),
            pid: 0,
        }
    }
}
