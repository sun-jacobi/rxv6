#![no_main]
#![no_std]

mod arch;
mod boot;
mod driver;
mod memory;

use crate::memory::{layout, vm::Kvm};
use core::panic::PanicInfo;
use memory::kalloc::Kalloc;
//====================================
#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    println!("{}", panic);
    loop {}
}

//====================================
#[no_mangle]
extern "C" fn kmain() {
    let mut uart = driver::uart::Uart::new();
    uart.init();
    // Kalloc::test();
    // initialize the linker script provided variable
    unsafe {
        layout::init_linker_variable();
    }
    let mut kalloc = Kalloc::new().kinit(); // init the kernel page allocator
    Kvm::init(&mut kalloc); // init the kernel page table
    Kvm::init_hart(); // turn on paging hardware
    let mut uart = driver::uart::Uart::new();
    uart.init();
    print!("\x1B[2J\x1B[1;1H");
    println!("rxv6: an eduacationol OS in Rust");
    println!("{}", LOGO);
}

pub static LOGO: &'static str = r"
 ▄▄▄▄▄▄▄▄▄▄▄  ▄       ▄  ▄               ▄  ▄▄▄▄▄▄▄▄▄▄▄ 
▐░░░░░░░░░░░▌▐░▌     ▐░▌▐░▌             ▐░▌▐░░░░░░░░░░░▌
▐░█▀▀▀▀▀▀▀█░▌ ▐░▌   ▐░▌  ▐░▌           ▐░▌ ▐░█▀▀▀▀▀▀▀▀▀ 
▐░▌       ▐░▌  ▐░▌ ▐░▌    ▐░▌         ▐░▌  ▐░▌          
▐░█▄▄▄▄▄▄▄█░▌   ▐░▐░▌      ▐░▌       ▐░▌   ▐░█▄▄▄▄▄▄▄▄▄ 
▐░░░░░░░░░░░▌    ▐░▌        ▐░▌     ▐░▌    ▐░░░░░░░░░░░▌
▐░█▀▀▀▀█░█▀▀    ▐░▌░▌        ▐░▌   ▐░▌     ▐░█▀▀▀▀▀▀▀█░▌
▐░▌     ▐░▌    ▐░▌ ▐░▌        ▐░▌ ▐░▌      ▐░▌       ▐░▌
▐░▌      ▐░▌  ▐░▌   ▐░▌        ▐░▐░▌       ▐░█▄▄▄▄▄▄▄█░▌
▐░▌       ▐░▌▐░▌     ▐░▌        ▐░▌        ▐░░░░░░░░░░░▌
 ▀         ▀  ▀       ▀          ▀          ▀▀▀▀▀▀▀▀▀▀▀ 
";
