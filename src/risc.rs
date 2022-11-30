pub(crate) mod tp {
    pub(crate) fn write(bits: usize) {
        use core::arch::asm;
        unsafe {
            asm!("mv tp, {}", in(reg) bits);
        }
    }
}

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

pub(crate) fn setup_sie() {
    use riscv::register;
    unsafe {
        register::sie::set_sext();
        register::sie::set_ssoft();
        register::sie::set_stimer();
    }
}
