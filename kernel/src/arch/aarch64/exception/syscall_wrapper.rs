use crate::{arch::{ThreadCtx, RegType}, addr_type::UserAddr, syscall::sys_write, println};


pub fn write_wrapper(ctx:&ThreadCtx)->i64{
    let fd=ctx[RegType::X0];
    let buf_len=ctx[RegType::X2];
    let buf_addr=UserAddr::new(ctx[RegType::X1]);
    sys_write(fd, buf_addr, buf_len)
}