use cortex_a::registers::*;
use tock_registers::{interfaces::Readable, fields::FieldValue};

core::arch::global_asm!(include_str!("exception_table.S"));

extern "C"{
    fn set_exception_vector();
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
        Some(ESR_EL1::EC::Value::SVC64) => syscall(),
        Some(ESR_EL1::EC::Value::DataAbortLowerEL) => adress_fault(),
        _ => exception_handler()
   }
}