use alloc::vec::Vec;
use crate::{UserAddr, scheduler::CURRENT_SCHEDULER, addr_type::Addr};
//ctrl+a 1
//..
//ctrl+d 4
//..
//ctrl+z 26

struct console{
    buf:[char;128],
    w:usize,
    r:usize,
    edit:usize
}

static mut CONS:console = console{buf:[0 as char;128],w:0,r:0,edit:0};

// use for echo
fn console_putc(ch:char){
    let ch = if ch == '\r' {'\n'} else {ch};
    if ch==127 as char{
        print!("\x08 \x08");
    }else{
        print!("{}",ch);
    }
}  

pub fn console_write(src_addr:UserAddr,src_len:u64){
    let mut buf:Vec<u8>=Vec::with_capacity(src_len as usize);
    buf.resize(src_len as usize, 0);
    match CURRENT_SCHEDULER.exclusive_access().cur_thread().unwrap().get_space().exclusive_access().read_from_space(&mut buf, src_addr.addr()){
        Ok(_)=>{
            for _c in buf{
                print!("{}",_c as char);
            }
        }
        Err(_) => {panic!("console_write fail!")},
    }
}

pub fn console_intr(ch:char){
    
}
