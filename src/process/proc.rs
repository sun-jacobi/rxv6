/// State of a Process
enum State {
    Unused,
    Used,
    Sleeping,
    Runnable,
    Running,
    Zombie,
}

// Per-process state
pub(crate) struct Proc {
    state: State,
    pid: u8,
    // kstack : u64,
}

impl Proc {
    pub(crate) fn pid(&self) -> u8 {
        self.pid
    }

    pub(crate) fn new() -> Self {
        Proc {
            state: State::Runnable,
            pid: 0,
        }
    }

    pub(crate) fn init(self) -> Self {
        self
    }
}

pub(crate) struct Scheduler {}

impl Scheduler {
    pub(crate) fn run(self) -> ! {
        loop {}
    }
}
