#![no_main]
#![no_std]
#[allow(clippy::borrow_interior_mutable_const)]
mod arch;
mod boot;
mod driver;
mod lock;
mod memory;
mod process;
mod trap;

use crate::{
    arch::cpu_id,
    memory::{layout, vm::Kvm},
};
use core::panic::PanicInfo;
use memory::kalloc::Kalloc;
use process::master::PMASTER;
//====================================
#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    println!("{}", panic);
    loop {}
}
//====================================
pub(crate) fn kmain() {
    if cpu_id() == 0 {
        let mut uart = driver::uart::Uart::new();
        uart.init();
        print!("\x1B[2J\x1B[1;1H");
        println!("RXV6: An Eduacationol OS In Rust.");
        println!("{}", LOGO);
        Kalloc::kinit(); // init the kernel page allocator.
        Kvm::init().init_hart(); // create and turn on the kernel page table.
        println!("Loading Kernel Page Table...");
        trap::init(); // install kernel trap vector
        trap::plic::init(); // set up interrupt controller
        trap::plic::init_hart(); // ask PLIC for device interrupts
        println!("Loading Kernel Trap and PLIC...");
        process::init(); // process table
        process::user_init(); // first user process
        println!("Entering Userland...");
    } else {
    }

    unsafe {
        PMASTER.scheduler();
    }
}

pub static LOGO: &str = r"
________     ___    ___ ___      ___ ________     
|\   __  \   |\  \  /  /|\  \    /  /|\   ____\    
\ \  \|\  \  \ \  \/  / | \  \  /  / | \  \___|    
 \ \   _  _\  \ \    / / \ \  \/  / / \ \  \____   
  \ \  \\  \|  /     \/   \ \    / /   \ \  ___  \ 
   \ \__\\ _\ /  /\   \    \ \__/ /     \ \_______\
    \|__|\|__/__/ /\ __\    \|__|/       \|_______|
             |__|/ \|__|                           
";
