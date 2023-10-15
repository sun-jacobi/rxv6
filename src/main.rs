#![no_main]
#![no_std]
#![feature(atomic_bool_fetch_not)]
#![feature(strict_provenance)]
#![feature(const_trait_impl)]

mod arch;
mod boot;
mod driver;
mod lock;
mod memory;
mod process;
mod syscall;
mod trap;

extern crate alloc;

use crate::{
    arch::cpu_id,
    memory::{heap, kmalloc::Kmalloc, layout, vm::Kvm},
};
use core::sync::atomic::Ordering::SeqCst;
use core::{
    hint::spin_loop,
    panic::PanicInfo,
    sync::atomic::{fence, AtomicBool},
};
use memory::kalloc::Kalloc;
use process::master::PMASTER;
//====================================
#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    println!("{}", panic);
    loop {}
}
//====================================
static STARTED: AtomicBool = AtomicBool::new(false);
pub(crate) fn kmain() {
    if cpu_id() == 0 {
        let mut uart = driver::uart::Uart::new();
        uart.init();
        print!("\x1B[2J\x1B[1;1H");
        println!("RXV6: An Eduacationol OS In Rust.");
        println!("{}", LOGO);
        Kalloc::init_kernel_page_allocator(); // init the kernel page allocator.
        Kvm::init_kernel_page_table(); // create the kernel page table.
        Kvm::init_hart(); // turn on the kernel page table.
        heap::init_kernel_heap(); // init the kernel heap
        println!("Loading Kernel Page Table...");
        trap::init(); // install kernel trap vector
        trap::plic::init(); // set up interrupt controller
        trap::plic::init_hart(); // ask PLIC for device interrupts
        println!("Loading Kernel Trap and PLIC...");
        process::init(); // process table
        process::user_init(); // first user process
        println!("Entering Userland...");
        STARTED.fetch_not(core::sync::atomic::Ordering::SeqCst);
    } else {
        while !STARTED.load(core::sync::atomic::Ordering::SeqCst) {
            spin_loop();
        }

        fence(SeqCst); // memory barrier

        Kvm::init_hart(); // turn on paging
        trap::init(); // install kernel trap vector
        trap::plic::init_hart(); // ask PLIC for device interrupts
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
