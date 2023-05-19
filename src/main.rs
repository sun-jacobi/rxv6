#![no_main]
#![no_std]

mod arch;
mod boot;
mod driver;
mod lock;
mod memory;
// mod process;
mod trap;

use crate::{
    arch::cpu_id,
    memory::{
        layout,
        vm::Kvm,
    },
};
use core::{hint::spin_loop, panic::PanicInfo, arch::asm};
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
    if cpu_id() == 0 {
        let mut uart = driver::uart::Uart::new();
        uart.init();
        print!("\x1B[2J\x1B[1;1H");
        println!("RXV6: An Eduacationol OS In Rust.");
        println!("{}", LOGO);
        Kalloc::kinit(); // init the kernel page allocator
        Kvm::init(); // init the kernel page table
        Kvm::init_hart(); // turn on paging hardware
        println!("Initialized Kernel Page Table.");
        // process::init(); // process table
        trap::init(); // install kernel trap vector
        trap::plic::init(); // set up interrupt controller
        trap::plic::init_hart(); // ask PLIC for device interrupts
        println!("Initialized PLIC.");
        loop {
            unsafe { asm!("wfi") };
        }
    } else {
        spin_loop()
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


