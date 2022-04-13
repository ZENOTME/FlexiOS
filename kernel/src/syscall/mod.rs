


use crate::{addr_type::{UserAddr}, driver};

const STDOUT:u64=1;
const STDIN:u64=0;
//64
pub fn sys_write(fd:u64,buf_addr:UserAddr,buf_len:u64)->i64{
    //println!("fd:{} buf_addr:{:?} buf_len:{}",fd,buf_addr,buf_len);
    // Standard Ouput
    match fd{
        STDOUT=>{driver::console::console_write(buf_addr, buf_len);}
        _=>{return -1;}
    }
    return 0;
}

pub fn sys_read(fd:u64,buf_addr:UserAddr,buf_len:u64)->i64{
    0
}