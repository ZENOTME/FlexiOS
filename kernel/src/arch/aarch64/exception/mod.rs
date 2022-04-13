use cortex_a::registers::*;
use tock_registers::interfaces::Readable;

use self::syscall_wrapper::syscall_router;

core::arch::global_asm!(include_str!("exception_table.S"));
core::arch::global_asm!(include_str!("irq.S"));

mod syscall_wrapper;

extern "C" {
    fn set_exception_vector();
    pub fn eret_to_user(sp: u64);
    pub fn enable_irq();
    pub fn disable_irq();
}

pub fn exception_init() {
    unsafe {
        set_exception_vector();
    }
}

#[no_mangle]
extern "C" fn irq_handler() {
    crate::driver::gic::gicvc2_handler();
    unsafe {
        enable_irq();
    }
}

#[no_mangle]
extern "C" fn frq_handler() {
    panic!("Unsupported frq!");
}

#[no_mangle]
extern "C" fn serror_handler() {
    panic!("Unsupported serror!");
}

#[no_mangle]
extern "C" fn syn_exception_handler(sp: u64) {
    let esr_ec = ESR_EL1.read_as_enum(ESR_EL1::EC);
    match esr_ec {
        Some(ESR_EL1::EC::Value::SVC64) => syscall_router(sp),
        Some(ESR_EL1::EC::Value::DataAbortLowerEL) => panic!("Unsupported DataAbortLowerEL"),
        Some(other) => panic!(
            "Other exception:{:#b} FaultAddr: {:#x} ExceptionLinkAddr: {:#x}",
            other as usize,
            FAR_EL1.get(),
            ELR_EL1.get()
        ),
        None => panic!("None"),
    }
}
