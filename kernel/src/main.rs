//! # Boot flow
//!
//! 1. The kernel's entry point is the function `cpu::boot::arch_boot::_start()`.
//!     - It is implemented in `src/_arch/__arch_name__/cpu/boot.s`.
//! 2. Once finished with architectural setup, the arch code calls `kernel_init()`.

#![no_main]
#![no_std]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(const_fn_trait_bound)]
#![feature(llvm_asm)]
#![feature(step_trait)]

use crate::{heap_allocator::init_heap, frame_allocator::CURRENT_FRAME_ALLOCATOR};

extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate tock_registers;
#[macro_use]
extern crate cortex_a;
#[macro_use]
extern crate virtio_drivers;


#[path = "arch/aarch64/mod.rs"]
pub mod arch;
#[macro_use] // print!
pub mod logging;
mod panic_wait;
mod consts;
mod up;
mod addr_type;
mod frame_allocator;
mod heap_allocator;
mod addr_space;
mod driver;
mod frame;
mod thread;
// Contemporary Loader

mod loader;

use core::arch::global_asm;
global_asm!(include_str!("link_app.S"));

pub fn kmain() -> ! {
    heap_allocator::init_heap();
    logging::init();
    println!(">>LOG test..");
    info!("info");
    warn!("warn");
    error!("error");
    println!(">>Information of Frame allocator");
    CURRENT_FRAME_ALLOCATOR.exclusive_access().print_state();
    println!(">>driver test");
    driver::driver_init();
    println!(">> List all app");
    loader::list_apps();
    
    
    panic!("Close OS!")
}
