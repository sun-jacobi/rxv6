#![no_std]
#![feature(panic_info_message)]

#[no_mangle]
extern "C" fn abort() -> ! {
    use core::arch::asm;
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    use core::arch::asm;
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

//=================================
// jump from start

mod kalloc;
mod memlayout;
mod proc;
mod risc;
mod spinlock;
mod start;
mod vm;
#[no_mangle]
extern "C" fn kmain() {
    use core::arch::asm;
    unsafe {
        asm!("li a3,2"); // a helper flag
        asm!("wfi");
    }
}
