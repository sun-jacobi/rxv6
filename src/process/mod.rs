pub(crate) mod cpu;
pub(crate) mod master;
pub(crate) mod proc;
use crate::process::master::PMASTER;

// initialize the process table
pub(crate) fn init() {
    unsafe {
        PMASTER.init();
    }
}
