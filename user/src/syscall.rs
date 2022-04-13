use core::arch::asm;

const SYSCALL_READ:usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_SWPAN:usize =170;


fn syscall(id: usize, args: [usize; 8]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "svc #2",
            inlateout("x0") args[0] => ret,
            in("x1") args[1],
            in("x2") args[2],
            in("x3") args[3],
            in("x4") args[4],
            in("x5") args[5],
            in("x6") args[6],
            in("x7") args[7],
            in("x8") id
        );
    }
    ret
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len(),0,0,0,0,0])
}

pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0,0,0,0,0,0])
}

pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPEN, [path.as_ptr() as usize, flags as usize, 0,0,0,0,0,0])
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0,0,0,0,0,0])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buffer.as_mut_ptr() as usize, buffer.len(),0,0,0,0,0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0,0,0,0,0,0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0,0,0,0,0,0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0,0,0,0,0,0])
}

pub fn sys_spawn(path:&str) -> isize{
    syscall(SYSCALL_SWPAN, [path.as_ptr() as usize,0, 0, 0,0,0,0,0])
}