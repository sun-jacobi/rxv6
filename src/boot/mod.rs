core::arch::global_asm!(include_str!("../asm/entry.S"));
core::arch::global_asm!(include_str!("../asm/kernelvec.S"));
core::arch::global_asm!(include_str!("../asm/switch.S"));
core::arch::global_asm!(include_str!("../asm/trampoline.S"));

pub(crate) mod start;
