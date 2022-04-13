use cortex_a::registers::{CNTFRQ_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use tock_registers::interfaces::{Readable, Writeable};
const TICK_MS: u64 = 500;
static mut cntv_tval: u64 = 0;

pub fn timer_init() {
    let frq = CNTFRQ_EL0.get();
    println!("frequency:{}", frq);
    let tval = frq * TICK_MS / 1000;
    unsafe {
        cntv_tval = tval;
    }
    CNTP_TVAL_EL0.set(tval);
    println!("set timer: {}", CNTP_TVAL_EL0.get());
}

fn plat_handle_timer_irq() {
    unsafe {
        CNTP_TVAL_EL0.set(cntv_tval);
    }
    CNTP_CTL_EL0
        .write(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR + CNTP_CTL_EL0::IMASK::CLEAR);
}

pub fn timer_irq() {
    plat_handle_timer_irq();
}

pub fn timer_enable() {
    CNTP_CTL_EL0
        .write(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR + CNTP_CTL_EL0::IMASK::CLEAR);
}

pub fn timer_disable() {
    CNTP_CTL_EL0.write(
        CNTP_CTL_EL0::ENABLE::CLEAR + CNTP_CTL_EL0::IMASK::CLEAR + CNTP_CTL_EL0::IMASK::CLEAR,
    );
}
