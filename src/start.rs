//=================================
// jump from entry.S
use crate::kmain;
use crate::risc;
use core::arch::asm;

static mut TIMER_SCRATCH: [[usize; 5]; 4] = [[0; 5]; 4];

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

    // delegate all interrupts and exceptions to supervisor mode.
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

#[allow(non_snake_case)]
fn timer_init() {
    use riscv::register;
    // each CPU has a separate source of timer interrupts.
    let id = register::mhartid::read();

    // (TODO): ask the CLINT for a timer interrupt.
    let interval = 1000000;

    // (TODO): prepare information in scratch[] for timervec.
    // scratch[0..2] : space for timervec to save registers.
    // scratch[3] : address of CLINT MTIMECMP register.
    // scratch[4] : desired interval (in cycles) between timer interrupts.

    let mut scratch = unsafe { TIMER_SCRATCH[id] };
    scratch[3] = unsafe {
        let CLINT = 0x2000000;
        let MTIMECMP = (CLINT + 0x4000 + 8 * (id)) as *mut usize;
        *MTIMECMP = CLINT + 0xBFF8 + interval;
        *MTIMECMP
    };
    scratch[4] = interval;
    register::mscratch::write(scratch.as_mut_ptr() as usize);

    // set the machine-mode trap handler
    let timevec = unsafe {
        let x: usize;
        asm!("la {0}, timervec", out(reg) x);
        x
    };
    unsafe {
        register::mtvec::write(timevec, register::mtvec::TrapMode::Direct);
    }

    unsafe {
        // enable machine-mode interrupts.
        register::mstatus::set_mie();
        // enable machine-mode timer interrupts.
        register::mie::set_mtimer();
    }
}
