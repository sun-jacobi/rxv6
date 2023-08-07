use core::ptr::write_volatile;

use riscv::register::sstatus;

use crate::arch::NCPU;
use crate::arch::{intr_off, intr_on};
use crate::cpu_id;
use crate::memory::kalloc::KALLOC;

// Saved registers for kernel context switches.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Context {
    pub(crate) ra: u64,
    pub(crate) sp: u64,

    // callee-saved register
    s0: u64,
    s1: u64,
    s2: u64,
    s3: u64,
    s4: u64,
    s5: u64,
    s6: u64,
    s7: u64,
    s8: u64,
    s9: u64,
    s10: u64,
    s11: u64,
}

// per-process data for the trap handling code in trampoline.S.
// sits in a page by itself just under the trampoline page in the
// user page table. not specially mapped in the kernel page table.
// uservec in trampoline.S saves user registers in the trapframe,
// then initializes registers from the trapframe's
// kernel_sp, kernel_hartid, kernel_satp, and jumps to kernel_trap.
// usertrapret() and userret in trampoline.S set up
// the trapframe's kernel_*, restore user registers from the
// trapframe, switch to the user page table, and enter user space.
// the trapframe includes callee-saved user registers like s0-s11 because the
// return-to-user path via usertrapret() doesn't return through
// the entire kernel call stack.
#[repr(C)]
#[derive(Default)]
pub(crate) struct TrapFrame {
    /*   0 */ pub(crate) kernel_satp: u64, // kernel page table
    /*   8 */ pub(crate) kernel_sp: u64, // top of process's kernel stack
    /*  16 */ pub(crate) kernel_trap: u64, // usertrap()
    /*  24 */ pub(crate) epc: u64, // saved user program counter
    /*  32 */ pub(crate) kernel_hartid: u64, // saved kernel tp
    /*  40 */ pub(crate) ra: u64,
    /*  48 */ pub(crate) sp: u64,
    /*  56 */ pub(crate) gp: u64,
    /*  64 */ pub(crate) tp: u64,
    /*  72 */ pub(crate) t0: u64,
    /*  80 */ pub(crate) t1: u64,
    /*  88 */ pub(crate) t2: u64,
    /*  96 */ pub(crate) s0: u64,
    /* 104 */ pub(crate) s1: u64,
    /* 112 */ pub(crate) a0: u64,
    /* 120 */ pub(crate) a1: u64,
    /* 128 */ pub(crate) a2: u64,
    /* 136 */ pub(crate) a3: u64,
    /* 144 */ pub(crate) a4: u64,
    /* 152 */ pub(crate) a5: u64,
    /* 160 */ pub(crate) a6: u64,
    /* 168 */ pub(crate) a7: u64,
    /* 176 */ pub(crate) s2: u64,
    /* 184 */ pub(crate) s3: u64,
    /* 192 */ pub(crate) s4: u64,
    /* 200 */ pub(crate) s5: u64,
    /* 208 */ pub(crate) s6: u64,
    /* 216 */ pub(crate) s7: u64,
    /* 224 */ pub(crate) s8: u64,
    /* 232 */ pub(crate) s9: u64,
    /* 240 */ pub(crate) s10: u64,
    /* 248 */ pub(crate) s11: u64,
    /* 256 */ pub(crate) t3: u64,
    /* 264 */ pub(crate) t4: u64,
    /* 272 */ pub(crate) t5: u64,
    /* 280 */ pub(crate) t6: u64,
}

impl TrapFrame {
    pub(crate) fn new() -> Option<*mut Self> {
        let frame = KALLOC.lock().alloc()? as *mut TrapFrame;
        unsafe {
            write_volatile(frame, TrapFrame::default());
        }
        Some(frame)
    }
}

impl Context {
    pub(crate) const fn default() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        }
    }
}

// Per-CPU state.
#[derive(Clone, Copy)]
pub(crate) struct CPU {
    pub(crate) context: Context,
    pub(crate) pin: Option<usize>, // index in process table
    pub(crate) nlock: u8,          // number of acquired lock
    pub(crate) intr: bool,
}

impl CPU {
    pub(crate) const fn new() -> Self {
        CPU {
            context: Context::default(),
            pin: None,
            nlock: 0,    // num of locks be used
            intr: false, // is interrupt already on
        }
    }
}

pub(crate) static mut CMASTER: CMaster = CMaster::new();

pub(crate) struct CMaster {
    cpus: [CPU; NCPU],
}

impl CMaster {
    const fn new() -> Self {
        Self {
            cpus: [CPU::new(), CPU::new(), CPU::new(), CPU::new()],
        }
    }

    pub(crate) fn my_cpu(&self) -> &CPU {
        let cpu_id = cpu_id();
        &self.cpus[cpu_id]
    }

    // Return this CPU's cpu struct.
    // Interrupts must be disabled.
    pub(crate) fn my_cpu_mut(&mut self) -> &mut CPU {
        let cpu_id = cpu_id();
        &mut self.cpus[cpu_id]
    }

    pub(crate) fn push_off(&mut self) {
        let old = sstatus::read().sie();
        intr_off(); // disable the interrupt to avoid the deadlock.
        let cpu = self.my_cpu_mut();
        cpu.intr = old;
        cpu.nlock += 1;
    }

    pub(crate) fn pop_off(&mut self) {
        let cpu = self.my_cpu_mut();
        assert_ne!(cpu.nlock, 0);
        cpu.nlock -= 1;
        if cpu.nlock == 0 && cpu.intr {
            intr_on();
        }
    }

    // Return the current pin for this cpu, or zero if none.
    pub(crate) fn my_proc(&mut self) -> usize {
        let pin = unsafe { CMASTER.my_cpu().pin.unwrap() };
        pin
    }
}
