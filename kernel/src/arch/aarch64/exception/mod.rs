use cortex_a::registers::*;
use tock_registers::{interfaces::Readable};

use crate::{println, scheduler::CURRENT_SCHEDULER, addr_type::Addr, arch::{ThreadCtx, RegType}, syscall::sys_write};

core::arch::global_asm!(include_str!("exception_table.S"));

mod syscall_wrapper;

extern "C"{
    fn set_exception_vector();
    pub fn eret_to_thread(sp:u64);
}

pub fn exception_init(){
    unsafe{set_exception_vector()};
}


#[no_mangle]
extern "C" fn exception_handler(){
    panic!("Unsupported exception!");
}

#[no_mangle]
extern "C" fn syn_exception_handler(sp:u64){
   let esr_ec=ESR_EL1.read_as_enum(ESR_EL1::EC);
   match esr_ec {
        Some(ESR_EL1::EC::Value::SVC64) => sys_call_router(sp),
        Some(ESR_EL1::EC::Value::DataAbortLowerEL) => panic!("Unsupported DataAbortLowerEL"),
        Some(other)=> panic!("Other exception:{:#b} FaultAddr: {:#x} ExceptionLinkAddr: {:#x}",other as usize,FAR_EL1.get(),ELR_EL1.get()),
        None=>panic!("None")
   }
}

pub fn sys_call_router(sp:u64){
    let ksp=CURRENT_SCHEDULER.exclusive_access().cur_thread().unwrap().get_kernel_stack();
    assert_eq!(sp,ksp.addr());
    let kernel_stack=unsafe{&*(ksp.addr() as *mut ThreadCtx)};
    let r=match kernel_stack[RegType::X8]{
        93=>panic!("Syscall exit"),
        64=>syscall_wrapper::write_wrapper(kernel_stack),
        _=>panic!("Unsupport Syscall type"),
    };
    let kernel_stack=unsafe{&mut *(ksp.addr() as *mut ThreadCtx)};
    kernel_stack[RegType::X0]=r as u64;
}




