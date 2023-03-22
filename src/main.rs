#![no_main]
#![no_std]

mod arch;
mod boot;
mod driver;
mod memory;

//====================================
use core::panic::PanicInfo;
use memory::kalloc::Kalloc;
#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    println!("panic");
    loop {}
}

#[no_mangle]
extern "C" fn kmain() {
    let mut uart = driver::uart::Uart::new(0x1000_0000);
    uart.init();
    println!("rxv6: an eduacationol OS in Rust");
    Kalloc::init(); // init the kernel page allocator
}
