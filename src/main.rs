#![no_main]
#![no_std]

mod arch;
mod boot;
mod driver;
mod memory;

//====================================
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn kmain() {
    let mut my_uart = driver::uart::Uart::new(0x1000_0000);

    my_uart.init();

    // Now test println! macro!
    println!("This is my operating system!");
}
