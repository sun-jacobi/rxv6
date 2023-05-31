use crate::arch::{cpu_id, intr_off, make_satp, w_sip, PGSIZE};
use crate::layout::TRAPTEXT;
use crate::memory::layout::{KERNELVEC, TRAMPOLINE};
use crate::process::cpu::CMASTER;
use crate::{PMASTER, print};
use riscv::register::{
    satp,
    scause::{self, Interrupt, Trap},
    sepc, sip,
    sstatus::{self, SPP},
    stvec,
    stvec::TrapMode,
};

pub(crate) mod plic;

pub(crate) fn init() {
    unsafe {
        riscv::register::stvec::write(KERNELVEC as usize, TrapMode::Direct);
    }
}

extern "C" {
    fn userret(pagetable: u64);
    fn uservec();
}

#[no_mangle]
extern "C" fn kerneltrap() {
    assert!(!sstatus::read().sie());
    match devintr() {
        // Software interrupt from a machine-mode timer interrupt.
        Interrupt::SupervisorSoft => {
            // acknowledge the software interrupt by clearing 
            // the SSIP bit in sip.
            w_sip(sip::read().bits() & !2);
            let pin = unsafe { CMASTER.my_cpu().pin };
            if pin != None {
                // give up the CPU.
                unsafe {
                    // PMASTER.step();
                }
            }
        }
        // Supervisor external interrupt
        Interrupt::SupervisorExternal => {
            unimplemented!("Interrupt::SupervisorExternal");
        }
        i => panic!("Kernel Panic: {:?} should not be handled in kernel", i),
    }
}

#[no_mangle]
extern "C" fn usertrap() {
    print!("{}", cpu_id());
    assert_eq!(sstatus::read().spp(), SPP::User);
    let p = unsafe { PMASTER.my_proc() };
    let trapframe = p.trapframe;
    unsafe {
        riscv::register::stvec::write(KERNELVEC as usize, TrapMode::Direct);
        (*trapframe).epc = sepc::read() as u64;
    }

    match devintr() {
        Interrupt::SupervisorSoft => unsafe {
            // timer interrupt
            PMASTER.step();
        },
        i => panic!("unsupported interrupt {:?}", i),
    }
    usertrapret();
}

// check if it's an external interrupt or software interrupt,
// and handle it.
fn devintr() -> Interrupt {
    let scause = scause::read().cause();
    match scause {
        Trap::Interrupt(i) => i,
        Trap::Exception(e) => panic!("Exception {:?}", e),
    }
}

// A fork child's very first scheduling by scheduler()
// will swtch to forkret.
pub(crate) fn forkret() {
    let p = unsafe { PMASTER.my_proc() };
    p.info.unlock();
    usertrapret();
}

// return to user space
pub(crate) fn usertrapret() {
    let p = unsafe { PMASTER.my_proc() }; 
    intr_off();
    // send syscalls, interrupts, and exceptions to uservec in trampoline.
    let trapframe = p.trapframe;

    // use uservec for supervisor interrupt 
    unsafe {
        let trampoline_uservec = TRAMPOLINE + (uservec as u64 - TRAPTEXT);
        stvec::write(trampoline_uservec as usize, TrapMode::Direct);
    }

    // set up trapframe values that uservec will need when
    // the process next traps into the kernel.
    unsafe {
        // kernel page table
        (*trapframe).kernel_satp = satp::read().bits() as u64;
        // process's kernel stack
        (*trapframe).kernel_sp = p.kstack.get().unwrap() + PGSIZE;
        (*trapframe).kernel_trap = usertrap as u64;
        // hartid
        (*trapframe).kernel_hartid = cpu_id() as u64;
    }

    // set up the registers that trampoline.S's sret will use
    // to get to user space.
    unsafe {
        // set S Previous Privilege mode to User
        sstatus::set_spp(SPP::User);
        sstatus::set_spie();
        // set S Exception Program Counter to the saved user pc.
        sepc::write((*p.trapframe).epc as usize);
    }
    // tell trampoline.S the user page table to switch to.
    let satp = make_satp(p.pagetable);
    let trampoline_userret_fn: extern "C" fn(satp: u64) = unsafe {
        let trampoline_userret_addr = TRAMPOLINE + (userret as u64 - TRAPTEXT);
        core::mem::transmute(trampoline_userret_addr)
    };
    trampoline_userret_fn(satp);
}
