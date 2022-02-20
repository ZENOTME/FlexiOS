const PL01_BASE:usize=0xffff_0000_0900_0000;
const CLOCK_BASE:usize=24000000;
const BAUDRATE:usize=115200;

const DR_OFFSET:usize=0x0000;
const FR_OFFSET:usize=0x018;
const IBRD_OFFSET:usize=0x024;
const FBRD_OFFSET:usize=0x028;
const LCR_OFFSET:usize=0x02c;
const CR_OFFSET:usize=0x030;
const IMSC_OFFSET:usize=0x038;
const DMACR_OFFSET:usize=0x048;

fn calculate_d(base_clock:usize,baudrate:usize)->(u32,u32){
    let div=4*base_clock/baudrate;
    (div as u32 &0x3f ,(div>>6) as u32 & 0xffff )
}


unsafe fn wait_tx_complete(){
    let FR_BUSY:u32=(1<<3);
    let ptr=PL01_BASE as *mut u32;
    while (ptr.add(FR_OFFSET).read_volatile()&FR_BUSY)!=0 {}
}

pub unsafe fn pl01_init(){
    let ptr = PL01_BASE as *mut u32;
    // Disable UART before anything else
    let cr = ptr.add(CR_OFFSET).read_volatile();
    ptr.add(CR_OFFSET).write_volatile(cr & 1);
    wait_tx_complete();
    // Flush FIFOs
    let lcr =ptr.add(LCR_OFFSET).read_volatile();
    ptr.add(LCR_OFFSET).write_volatile(lcr & !(1<<4));
    // Set frequency divisors (UARTIBRD and UARTFBRD) to configure the speed
    let brd=calculate_d(CLOCK_BASE, BAUDRATE);
    ptr.add(FBRD_OFFSET).write_volatile(brd.0);
    ptr.add(IBRD_OFFSET).write_volatile(brd.1);
    // Config the data format and stop bit: data format:8 stop bit 1
    // Mask all interrupts by setting corresponding bits to 1
    ptr.add(IMSC_OFFSET).write_volatile(0x7ff);
    // Disable DMA by setting all bits to 0
    ptr.add(DMACR_OFFSET).write_volatile(0);
    // Finally enable UART
    ptr.add(CR_OFFSET).write_volatile(1<<8 | 1);
}

pub unsafe fn pl01_send(ch:char)
{
    let ptr = PL01_BASE as *mut u32;
    wait_tx_complete();
    ptr.add(DR_OFFSET).write_volatile(ch as u32);
}