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

#[no_mangle]
extern "C" fn kmain() {
    let _kvm = vm::KVM::new();
    loop {}
}

//================================
// mod imported
mod address;
mod kalloc;
mod memlayout;
mod proc;
mod risc;
mod start;
mod vm;
