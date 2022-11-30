//=================================
// jump from entry.S

use crate::kmain;
use crate::risc;
use core::arch::asm;
#[allow(dead_code)]
#[no_mangle]
extern "C" fn start() {
    use riscv::register;

    // set M Previous Privilege mode to Supervisor, for mret.
    unsafe {
        register::mstatus::set_mpp(register::mstatus::MPP::Supervisor);
    }
    // set M Exception Program Counter to main, for mret.
    register::mepc::write(kmain as usize);

    register::satp::write(0); // disable the page table for now

    // Tdelegate all interrupts and exceptions to supervisor mode.
    risc::setup_medeleg();
    risc::setup_mideleg();
    risc::setup_sie();

    // configure Physical Memory Protection to give supervisor mode
    // access to all of physical memory.
    register::pmpaddr0::write(0x3fffffffffffff);
    register::pmpcfg0::write(0xf);

    // ask for clock interrupts.
    timer_init();

    // keep each CPU's hartid in its tp register, for cpuid().
    let hart_id = register::mhartid::read();
    risc::tp::write(hart_id);

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
