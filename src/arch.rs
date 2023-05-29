use core::{
    arch::asm,
    ptr::{read_volatile, write_volatile},
};
use riscv::register::{self, sstatus};

use crate::memory::layout::{CLINT, CLINT_MTIME};

pub(crate) const MAXVA: u64 = 1 << (9 + 9 + 9 + 12 - 1); // sv39
pub(crate) const PGSIZE: u64 = 4096; // 4096 bytes;
pub(crate) const NCPU: usize = 4; // number of cpus
pub(crate) const INTERVAL: u64 = 1000000; // about 1/10th second in qemu.
pub(crate) const NPROC: usize = 64; // maximum number of processes

pub(crate) mod tp {
    #[inline]
    pub(crate) fn write(bits: usize) {
        unsafe {
            core::arch::asm!("mv tp, {}", in(reg) bits);
        }
    }
}

#[inline]
pub(crate) fn disable_vm() {
    register::satp::write(0);
}

pub(crate) fn config_pm_protection() {
    register::pmpaddr0::write(0x3fffffffffffff);
    register::pmpcfg0::write(0xf);
}

#[inline]
pub(crate) fn setup_medeleg() {
    use riscv::register::medeleg::*;
    unsafe {
        set_breakpoint(); // Breakpoint Delegate
        set_illegal_instruction(); //Illegal Instruction Delegate
        set_instruction_fault(); //Instruction Access Fault Delegate
        set_instruction_misaligned(); //Instruction Address Misaligned Delegate
        set_instruction_page_fault(); //Instruction Page Fault Delegate
        set_load_fault(); //Load Access Fault Delegate
        set_load_misaligned(); //Load Address Misaligned Delegate
        set_load_page_fault(); //Load Page Fault Delegate
        set_machine_env_call(); //Environment Call from M-mode Delegate
        set_store_fault(); //Store/AMO Access fault
        set_store_misaligned(); //Store/AMO Address Misaligned Delegate
        set_store_page_fault(); //Store/AMO Page Fault Delegate
        set_supervisor_env_call(); //Environment Call from S-mode Delegate
        set_user_env_call(); //Environment Call from U-mode Delegate
    }
}

#[inline]
pub(crate) fn setup_mideleg() {
    use riscv::register::mideleg::*;
    unsafe {
        set_sext(); // Supervisor External Interrupt Delegate
        set_ssoft(); //Supervisor Software Interrupt Delegate
        set_stimer(); //Supervisor Timer Interrupt Delegate
        set_uext(); //User External Interrupt Delegate
        set_usoft(); //User Software Interrupt Delegate
        set_utimer(); //User Timer Interrupt Delegate
    }
}

#[inline]
pub(crate) fn setup_sie() {
    unsafe {
        register::sie::set_sext();
        register::sie::set_ssoft();
        register::sie::set_stimer();
    }
}

// scratch[0..2] : space for timervec to save registers.
// scratch[3] : address of CLINT MTIMECMP register.
// scratch[4] : desired interval (in cycles) between timer interrupts.
pub(crate) unsafe fn setup_mscratch() {
    use crate::memory::layout::SCRATCH;
    // each CPU has a separate source of timer interrupts.
    let id = register::mhartid::read();
    // ask the CLINT for a timer interrupt.
    let mtimecmp_val = unsafe { get_clint_mtime() + INTERVAL };
    let mtimecmp_addr = get_mtimecmp_addr(id);
    write_volatile(mtimecmp_addr as *mut u64, mtimecmp_val);
    SCRATCH[id][3] = mtimecmp_addr;
    SCRATCH[id][4] = INTERVAL;
    register::mscratch::write(SCRATCH[id].as_ptr() as usize);
}

#[inline]
unsafe fn get_clint_mtime() -> u64 {
    let ptr = CLINT_MTIME as *const u64;
    read_volatile(ptr)
}

#[inline]
fn get_mtimecmp_addr(cpu_id: usize) -> u64 {
    CLINT + 0x4000 + 8 * cpu_id as u64
}

#[inline]
pub(crate) fn mret() {
    unsafe {
        asm!("mret");
    }
}

#[inline]
pub(crate) fn cpu_id() -> usize {
    let id: usize;
    unsafe {
        asm!("mv {id}, tp", id = out(reg) id);
    }
    id
}

#[inline]
pub(crate) fn intr_on() {
    unsafe {
        sstatus::set_sie();
    }
}

#[inline]
pub(crate) fn intr_off() {
    unsafe {
        sstatus::clear_sie();
    }
}

pub(crate) fn make_satp(addr: u64) -> u64 {
    (addr >> 12) | (8 << 60)
}

#[inline]
pub(crate) fn w_sip(bits: usize) {
    unsafe {
        asm!("csrw sip, {}", in(reg) bits);
    }
}
