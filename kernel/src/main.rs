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
#![feature(asm)]
#![feature(step_trait)]

use crate::heap_allocator::init_heap;

extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate tock_registers;
#[macro_use]
extern crate cortex_a;



#[path = "arch/aarch64/mod.rs"]
pub mod arch;
#[macro_use] // print!
pub mod logging;
mod panic_wait;
mod consts;
mod up;
mod mm_type;
mod frame_allocator;
mod heap_allocator;
mod addr_space;
/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
pub fn kmain() -> ! {
    println!("[0] Hello My OS!");
    println!("[1] Initing...");
    heap_allocator::init_heap();
    println!("[2] Testing...");
    addr_space::test_solve();
    panic!("Close OS!")
}
