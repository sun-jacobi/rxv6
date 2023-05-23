use core::hint::spin_loop;

use crate::PMASTER;
use crate::{memory::layout::KERNELVEC, print, println};
use riscv::register::{
    scause::{self, Interrupt, Trap},
    utvec::TrapMode,
};

pub(crate) mod plic;

pub(crate) fn init() {
    unsafe {
        riscv::register::stvec::write(KERNELVEC as usize, TrapMode::Direct);
    }
}

#[no_mangle]
extern "C" fn kerneltrap() {
    match devintr() {
        // Software interrupt from a machine-mode timer interrupt.
        // intr -> kt --> sh -> kt -> continue
        Interrupt::SupervisorSoft => {
            unsafe {
                PMASTER.step();
            } // give up the CPU.
            return;
        }
        // Supervisor external interrupt
        Interrupt::SupervisorExternal => {
            unimplemented!("Interrupt::SupervisorExternal");
        }
        i => panic!("Kernel Panic: {:?} should not be handled in kernel", i),
    }
}

#[no_mangle]
extern "C" fn usertrap() {}

// check if it's an external interrupt or software interrupt,
// and handle it.
fn devintr() -> Interrupt {
    let scause = scause::read().cause();
    match scause {
        Trap::Interrupt(i) => i,
        Trap::Exception(e) => panic!("Kernel Panic: {:?}", e),
    }
}

// A fork child's very first scheduling by scheduler()
// will swtch to forkret.
pub(crate) fn _forkret() {}

pub(crate) fn usertrapret() {
    println!("USER TRAP RET");
    loop {
        spin_loop();
    }
}
