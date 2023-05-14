//=================================
// jump from entry.S

use core::arch::asm;

use crate::arch::{
    config_pm_protection, disable_vm, mret, setup_medeleg, setup_mideleg, setup_mscratch,
    setup_sie, tp,
};
use crate::layout::init_linker_variable;
use crate::memory::layout::TIMERVEC;
use crate::{kmain, print, println};
use riscv::register;
use riscv::register::utvec::TrapMode;

#[no_mangle]
extern "C" fn start() {
    // initialize the linker script provided variable
    unsafe {
        init_linker_variable();
    }
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

    // ask for clock interrupts.
    unsafe {
        timer_init();
    }

    // keep each CPU's hartid in its tp register, for cpuid().
    let hart_id = register::mhartid::read();
    tp::write(hart_id);

    // jump to kmain
    mret();
}

// arrange to receive timer interrupts.
// they will arrive in machine mode at
// at timervec in kernelvec.S,
// which turns them into software interrupts for
// devintr() in trap.c.
unsafe fn timer_init() {
    // prepare information in scratch[] for timervec.
    setup_mscratch();

    // set the machine-mode trap handler.
    register::mtvec::write(TIMERVEC as usize, TrapMode::Direct);

    // enable machine-mode interrupts.
    register::mstatus::set_mie();

    // enable machine-mode timer interrupts.
    register::mie::set_mtimer();
}

#[no_mangle]
extern "C" fn kerneltrap() {
    println!("Timer Interrupt Done");
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}
