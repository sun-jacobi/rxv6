pub(crate) mod cpu;
pub(crate) mod master;
pub(crate) mod proc;
use crate::process::master::MASTER;

// initialize the process table
pub(crate) fn init() {
    MASTER.init();
}
