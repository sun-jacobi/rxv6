//=================================
// jump from entry.S

use core::arch::asm;
#[allow(dead_code)]
#[no_mangle]
extern "C" fn start() {
    use riscv::register;

    unsafe {
        // set M Previous Privilege mode to Supervisor, for mret.
        register::mstatus::set_mpp(register::mstatus::MPP::Supervisor);
        // TODO: set M Exception Program Counter to main, for mret.
    }
    register::satp::write(0); // disable the page table

    // TODO : delegate all interrupts and exceptions to supervisor mode.

    // configure Physical Memory Protection to give supervisor mode
    // access to all of physical memory.
    timer_init();

    // keep each CPU's hartid in its tp register, for cpuid().

    // jump to kmain
    unsafe {
        asm!("mret");
    }
}

fn timer_init() {
    use riscv::register;
    // each CPU has a separate source of timer interrupts.
    let _id = register::mhartid::read();

    // TODO...
}
