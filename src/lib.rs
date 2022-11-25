#![no_std]
#[no_mangle]
extern "C" fn abort() -> ! {
    use core::arch::asm;
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

//=================================
// jump from start
mod risc;
mod start;
#[no_mangle]
extern "C" fn kmain() {}
