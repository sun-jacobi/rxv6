use crate::process::cpu::TrapFrame;

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
}

pub(crate) fn handle(trapframe: *mut TrapFrame) {
    unsafe {
        // sepc points to the ecall instruction,
        // but we want to return to the next instruction.
        (*trapframe).epc += 4;
    }

    let syscall_id = SysCall::from_trapframe(trapframe);
    match syscall_id {
        _ => unimplemented!("unimplemented syscall"),
    }
}

impl SysCall {
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
            _ => panic!("unsupported syscall"),
        }
    }
}
