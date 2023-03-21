#![no_main]
#![no_std]

mod arch;
mod boot;
mod driver;
mod memory;

//====================================
use core::panic::PanicInfo;
core::arch::global_asm!(include_str!("asm/entry.S"));
core::arch::global_asm!(include_str!("asm/kernelvec.S"));
core::arch::global_asm!(include_str!("asm/trampoline.S"));
core::arch::global_asm!(include_str!("asm/switch.S"));

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn kmain() {}
