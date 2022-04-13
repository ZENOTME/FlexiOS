use super::console::console_intr;

const PL01_BASE: usize = 0xffff_0000_0900_0000;
const CLOCK_BASE: usize = 24000000;
const BAUDRATE: usize = 115200;

// Data register
const DR_OFFSET: usize = 0x0000;
// Receive status / error clear
// const RSR_OFFSET:usize=0x004;
// Flag register
const FR_OFFSET: usize = 0x018;
// Baud rat
const IBRD_OFFSET: usize = 0x024;
const FBRD_OFFSET: usize = 0x028;
// Line control register
const LCR_OFFSET: usize = 0x02c;
// Control register
const CR_OFFSET: usize = 0x030;
// interrupt mask set/clear
const IMSC_OFFSET: usize = 0x038;
// masked interrupt status register
const MIS_OFFSET: usize = 0x040;
// interrupt clear registyer
const ICR_OFFSET: usize = 0x044;
// dma control register
const DMACR_OFFSET: usize = 0x048;

fn calculate_d(base_clock: usize, baudrate: usize) -> (u32, u32) {
    let div = 4 * base_clock / baudrate;
    (div as u32 & 0x3f, (div >> 6) as u32 & 0xffff)
}
#[inline]
fn read(reg_offset: usize) -> u32 {
    let addr = PL01_BASE + reg_offset;
    unsafe { (addr as *const u32).read_volatile() }
}
#[inline]
fn write(reg_offset: usize, val: u32) {
    wait_tx_complete();
    let addr = PL01_BASE + reg_offset;
    unsafe {
        (addr as *mut u32).write_volatile(val);
    }
}
fn wait_tx_complete() {
    loop {
        let stat = read(FR_OFFSET);
        if stat & 0x20 == 0 {
            break;
        }
    }
}

pub fn pl01_init() {
    // Disable UART before anything else
    let cr = read(CR_OFFSET);
    write(CR_OFFSET, cr & 1);
    // Enable FIFOs
    let lcr = read(LCR_OFFSET);
    write(LCR_OFFSET, lcr | 1 << 4);
    // Set frequency divisors (UARTIBRD and UARTFBRD) to configure the speed
    let brd = calculate_d(CLOCK_BASE, BAUDRATE);
    write(FBRD_OFFSET, brd.0);
    write(FBRD_OFFSET, brd.1);
    // Mask all interrupts by setting corresponding bits to 1
    //write(IMSC_OFFSET,0x7ff);
    write(IMSC_OFFSET, 0x7ff);
    // Disable DMA by setting all bits to 0
    write(DMACR_OFFSET, 0x1);
    // Finally enable UART
    write(CR_OFFSET, 1 << 8 | 1);
}

pub fn pl01_send(ch: char) {
    write(DR_OFFSET, ch as u32);
}

pub fn pl01_recv() -> u8 {
    while read(FR_OFFSET) & 0x10 != 0 {}
    let data = read(DR_OFFSET);
    data as u8
}

pub fn pl011_irq_handler() {
    let mis = read(MIS_OFFSET);
    while read(FR_OFFSET) & 0x10 == 0 {
        let data = read(DR_OFFSET) as u8;
        console_intr(data as char);
    }
    write(ICR_OFFSET, mis);
}
