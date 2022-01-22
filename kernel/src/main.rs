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

extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;


#[path = "arch/aarch64/mod.rs"]
pub mod arch;
#[macro_use] // print!
pub mod logging;
mod panic_wait;
mod consts;
mod mm;

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
pub fn kmain() -> ! {
    println!("[0] Hello from Rust!");
    println!("[1] init!");
    mm::init();
    println!("[2] heap test!");
    
    panic!("Stopping here.")
}
