use cortex_a::registers::*;
use tock_registers::{interfaces::Readable};

core::arch::global_asm!(include_str!("exception_table.S"));

extern "C"{
    fn set_exception_vector();
    pub fn eret_to_thread(sp:usize);
}

pub fn exception_init(){
    unsafe{set_exception_vector()};
}


#[no_mangle]
extern "C" fn exception_handler(){
    panic!("Unsupported exception!");
}

#[no_mangle]
extern "C" fn syn_exception_handler(){
   let esr_ec=ESR_EL1.read_as_enum(ESR_EL1::EC);
   match esr_ec {
        Some(ESR_EL1::EC::Value::SVC64) => exception_handler(),
        Some(ESR_EL1::EC::Value::DataAbortLowerEL) => exception_handler(),
        _ => exception_handler()
   }
}

