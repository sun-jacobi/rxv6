//=================================
// jump from entry.S

use crate::arch::{
    config_pm_protection, disable_vm, mret, setup_medeleg, setup_mideleg, setup_sie, tp,
};
use crate::kmain;
use riscv::register;

#[no_mangle]
extern "C" fn start() {
    // set M Previous Privilege mode to Supervisor, for mret.
    unsafe {
        register::mstatus::set_mpp(register::mstatus::MPP::Supervisor);
    }
    // set M Exception Program Counter to main, for mret.
    register::mepc::write(kmain as usize);

    disable_vm(); // disable the page table for now

    // delegate all interrupts and exceptions to supervisor mode.
    setup_medeleg();
    setup_mideleg();
    setup_sie();

    // configure Physical Memory Protection to give supervisor mode
    // access to all of physical memory.
    config_pm_protection();

    // ask for clock interrupts. (unimplemented now)
    // timer_init();

    // keep each CPU's hartid in its tp register, for cpuid().
    let hart_id = register::mhartid::read();
    tp::write(hart_id);

    // jump to kmain
    mret();
}
