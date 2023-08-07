use crate::{print, println, process::cpu::TrapFrame};

#[allow(dead_code)]
pub(crate) enum SysCall {
    Fork,
    Exit,
    Wait,
    Pipe,
    Read,
    Kill,
    Exec,
    Fstat,
    Chdir,
    Dup,
    GetPid,
    Sbrk,
    Sleep,
    Uptime,
    Open,
    Write,
    Mknod,
    Unlink,
    Link,
    Mkdir,
    Close,
    Log, // log for test
}

pub(crate) fn handle(trapframe: *mut TrapFrame) {
    unsafe {
        // sepc points to the ecall instruction,
        // but we want to return to the next instruction.
        (*trapframe).epc += 4;
    }

    let syscall = SysCall::from_trapframe(trapframe);

    match syscall {
        SysCall::Log => {
            test_log(trapframe);
        }
        _ => unimplemented!("unimplemented syscall"),
    }
}

impl SysCall {
    unsafe fn nth_arg(trapframe: *mut TrapFrame, n: u8) -> u64 {
        match n {
            0 => (*trapframe).a0,
            1 => (*trapframe).a1,
            2 => (*trapframe).a2,
            3 => (*trapframe).a3,
            4 => (*trapframe).a4,
            5 => (*trapframe).a5,
            _ => panic!("unsupported syscall argument"),
        }
    }

    fn from_trapframe(trapframe: *mut TrapFrame) -> SysCall {
        match unsafe { (*trapframe).a7 } {
            1 => SysCall::Fork,
            2 => SysCall::Exit,
            3 => SysCall::Wait,
            4 => SysCall::Pipe,
            5 => SysCall::Read,
            6 => SysCall::Kill,
            7 => SysCall::Exec,
            8 => SysCall::Fstat,
            9 => SysCall::Chdir,
            10 => SysCall::Dup,
            11 => SysCall::GetPid,
            12 => SysCall::Sbrk,
            13 => SysCall::Sleep,
            14 => SysCall::Uptime,
            15 => SysCall::Open,
            16 => SysCall::Write,
            17 => SysCall::Mknod,
            18 => SysCall::Unlink,
            19 => SysCall::Link,
            20 => SysCall::Mkdir,
            21 => SysCall::Close,
            22 => SysCall::Log,
            _ => panic!("unsupported syscall"),
        }
    }
}

fn test_log(trapframe: *mut TrapFrame) {
    let a0 = unsafe { SysCall::nth_arg(trapframe, 0) };
    println!("HELLO SYSCALL ARG  {}", a0);
}
