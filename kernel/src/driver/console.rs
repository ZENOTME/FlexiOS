use crate::{
    addr_type::Addr,
    thread::{CURRENT_CPU, CURRENT_SCHEDULER},
    UserAddr,
};
use alloc::vec::Vec;
use crossbeam_queue::ArrayQueue;
//ctrl+a 1
//..
//ctrl+d 4
//..
//ctrl+z 26
const QUEUE_LEN: usize = 128;
struct Console {
    queue: ArrayQueue<char>,
}

lazy_static::lazy_static! {
    static ref CONS: Console = Console{
        queue:ArrayQueue::new(QUEUE_LEN),
    };
}

// use for echo
fn console_putc(ch: char) {
    let ch = if ch == '\r' { '\n' } else { ch };
    if ch == 127 as char {
        print!("\x08 \x08");
    } else {
        print!("{}", ch);
    }
}

pub fn console_write(src_addr: UserAddr, src_len: u64) {
    let mut buf: Vec<u8> = Vec::with_capacity(src_len as usize);
    buf.resize(src_len as usize, 0);
    match CURRENT_CPU
        .try_get()
        .expect("No init")
        .exclusive_access()
        .cur_thread()
        .get_space()
        .exclusive_access()
        .read_from_space(&mut buf, src_addr.addr())
    {
        Ok(_) => {
            for _c in buf {
                print!("{}", _c as char);
            }
        }
        Err(_) => {
            panic!("console_write fail!")
        }
    }
}

pub fn console_intr(ch: char) {
    console_putc(ch);
    if !ch.is_control() {
        if let Err(_) = CONS.queue.push(ch) {
            panic!("consoles queue is full!");
        }
    }
}
