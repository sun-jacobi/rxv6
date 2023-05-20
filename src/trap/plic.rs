use core::ptr::write_volatile;

use crate::{
    arch::cpu_id,
    memory::layout::{PLIC, UART0_IRQ, VIRTIO0_IRQ},
};

pub(crate) fn init() {
    let plic_uart_addr = (PLIC + UART0_IRQ * 4) as *mut u32;
    let plic_virt_addr = (PLIC + VIRTIO0_IRQ * 4) as *mut u32;
    unsafe {
        write_volatile(plic_uart_addr, 1);
        write_volatile(plic_virt_addr, 1);
    }
}

pub(crate) fn init_hart() {
    let id = cpu_id() as u64;
    let plic_senable = (PLIC + 0x2080 + 0x100 * id) as *mut u32;
    let plic_spriority = (PLIC + 0x201000 + 0x2000 * id) as *mut u32;
    unsafe {
        write_volatile(plic_senable, (1 << UART0_IRQ) | (1 << VIRTIO0_IRQ));
        write_volatile(plic_spriority, 0);
    }
}
