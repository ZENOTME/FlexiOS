// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! Conditional reexporting of Board Support Packages.

//#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
#[path = "board/raspi/mod.rs"]
pub mod board;
pub mod consts;
pub mod cpu;
pub mod mm_type;
pub mod paging;
mod boot;
mod sched;
mod exception;

pub use sched::*;
pub use mm_type::*;
pub use exception::{eret_to_thread,enable_irq,disable_irq};

/// The entry point of kernel
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn master_main() -> ! {
    unsafe{exception::disable_irq();}
    exception::exception_init();
    crate::kmain();
}
