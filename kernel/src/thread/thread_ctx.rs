core::arch::global_asm!(include_str!("thread_swtch.S"));
/*
X19 = 0,
X20 = 1,
X21 = 2,
X22 = 3,
X23 = 4,
X24 = 5,
X25 = 6,
X26 = 7,
X27 = 8,
X28 = 9,
X30 = 10, //LR
SP = 11
*/

use crate::{arch::switch_to_user, thread::cpu::CURRENT_CPU};

use super::ThreadType;

#[repr(C)]
pub struct ThreadCtx {
    pub reg: [u64; 12],
}

impl ThreadCtx {
    pub fn new() -> Self {
        Self { reg: [0; 12] }
    }
    pub fn set(&mut self, stack_addr: u64, _type: ThreadType, entry: u64) {
        //set stack
        self.reg[11] = stack_addr;
        //set LR
        match _type {
            ThreadType::USER => self.reg[10] = first_ret as *const () as u64,
            ThreadType::KERNEL => self.reg[10] = entry,
        }
    }
    pub fn get_raw_addr(&self) -> u64 {
        self.reg.as_ptr() as u64
    }
}

extern "C" {
    pub fn thread_swtch(src: u64, dst: u64);
}

// first return
// for the user thread
pub extern "C" fn first_ret() {
    println!("Enter first ret");
    //switch to user mode
    let init_sp_addr = CURRENT_CPU
        .try_get()
        .expect("No init!")
        .exclusive_access()
        .cur_thread
        .as_ref()
        .unwrap()
        .get_kernel_stack();
    switch_to_user(init_sp_addr);
}
