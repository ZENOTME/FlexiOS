use crate::{
    addr_type::{Addr, UserAddr},
    arch::{RegType, UserCtx},
    syscall::{sys_read, sys_write},
    thread::CURRENT_CPU,
};

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_SWPAN: usize = 170;

pub fn syscall_router(sp: u64) {
    let ksp = CURRENT_CPU
        .try_get()
        .expect("No init")
        .exclusive_access()
        .cur_thread()
        .get_kernel_stack();
    assert_eq!(sp, ksp.addr());
    let kernel_stack = unsafe { &*(ksp.addr() as *mut UserCtx) };
    let r = match kernel_stack[RegType::X8] as usize {
        SYSCALL_EXIT => panic!("Syscall exit"),
        SYSCALL_WRITE => write_wrapper(kernel_stack),
        SYSCALL_READ => read_wraper(kernel_stack),
        _ => panic!("Unsupport Syscall type"),
    };
    let kernel_stack = unsafe { &mut *(ksp.addr() as *mut UserCtx) };
    kernel_stack[RegType::X0] = r as u64;
}

pub fn write_wrapper(ctx: &UserCtx) -> i64 {
    let fd = ctx[RegType::X0];
    let buf_len = ctx[RegType::X2];
    let buf_addr = UserAddr::new(ctx[RegType::X1]);
    sys_write(fd, buf_addr, buf_len)
}

pub fn read_wraper(ctx: &UserCtx) -> i64 {
    let fd = ctx[RegType::X0];
    let buf_len = ctx[RegType::X2];
    let buf_addr = UserAddr::new(ctx[RegType::X1]);
    sys_read(fd, buf_addr, buf_len)
}
