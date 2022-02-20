


use alloc::vec::{ *};

use crate::{addr_type::{UserAddr, Addr}, scheduler::CURRENT_SCHEDULER, driver::pl01_send};

//64
pub fn sys_write(fd:u64,buf_addr:UserAddr,buf_len:u64)->i64{
    //println!("fd:{} buf_addr:{:?} buf_len:{}",fd,buf_addr,buf_len);
    // Standard Ouput
    if fd!=1 {return -1;}
    let mut buf:Vec<u8>=Vec::with_capacity(buf_len as usize);
    buf.resize(buf_len as usize, 0);
    match CURRENT_SCHEDULER.exclusive_access().cur_thread().unwrap().get_space().exclusive_access().read_from_space(&mut buf, buf_addr.addr()){
        Ok(_)=>{
            for _c in buf{
                unsafe{pl01_send(_c as char)};
            }
        }
        Err(_) => {return -1;},
    }
    return 0;
}