#![no_main]
#![no_std]

mod arch;
mod boot;
mod driver;
mod lock;
mod memory;

use crate::{
    arch::set_plic_spriority,
    memory::{
        layout::{self, KERNELVEC},
        vm::Kvm,
    },
};
use core::{arch::asm, panic::PanicInfo};
use memory::kalloc::Kalloc;
use riscv::register::utvec::TrapMode;
//====================================
#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    println!("{}", panic);
    loop {}
}
//====================================
#[no_mangle]
extern "C" fn kmain() {
    Kalloc::kinit(); // init the kernel page allocator
    Kvm::init(); // init the kernel page table
    Kvm::init_hart(); // turn on paging hardware
    let mut uart = driver::uart::Uart::new();
    uart.init();
    print!("\x1B[2J\x1B[1;1H");
    println!("RXV6: An Eduacationol OS In Rust");
    println!("{}", LOGO);
    println!("Page Table Done.");
    unsafe {
        riscv::register::stvec::write(KERNELVEC as usize, TrapMode::Direct);
        set_plic_spriority();
    }
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

pub static LOGO: &'static str = r"
________     ___    ___ ___      ___ ________     
|\   __  \   |\  \  /  /|\  \    /  /|\   ____\    
\ \  \|\  \  \ \  \/  / | \  \  /  / | \  \___|    
 \ \   _  _\  \ \    / / \ \  \/  / / \ \  \____   
  \ \  \\  \|  /     \/   \ \    / /   \ \  ___  \ 
   \ \__\\ _\ /  /\   \    \ \__/ /     \ \_______\
    \|__|\|__/__/ /\ __\    \|__|/       \|_______|
             |__|/ \|__|                           
";
