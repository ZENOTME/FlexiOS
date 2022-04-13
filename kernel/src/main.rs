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

use tock_registers::interfaces::Readable;

use crate::{
    addr_type::{Addr, PageTableFlags, PhysAddr, UserAddr},
    arch::enable_irq,
    consts::{ROOT_THREAD_STACK_BASE, ROOT_THREAD_STACK_SIZE},
    driver::pl01_send,
    frame_allocator::CURRENT_FRAME_ALLOCATOR,
    heap_allocator::init_heap,
    thread::{SimpleScheduler, CPU, CURRENT_CPU, CURRENT_SCHEDULER},
    up::UPSafeCell,
};

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
mod addr_space;
mod addr_type;
mod consts;
mod driver;
mod frame;
mod frame_allocator;
mod heap_allocator;
mod panic_wait;
mod syscall;
mod thread;
mod up;
// Contemporary Loader

mod loader;

use core::arch::global_asm;
global_asm!(include_str!("link_app.S"));

pub fn kmain() -> ! {
    println!("Enter into kernel!");
    arch::switch_to_vmspace(PhysAddr::new(0));
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
    unsafe {
        println!(">> Init CPU structure");
        CURRENT_CPU
            .try_init_once(|| UPSafeCell::new(CPU::new()))
            .expect("Only init once");
        println!(">> Init scheduler");
        CURRENT_SCHEDULER
            .try_init_once(|| UPSafeCell::new(SimpleScheduler::new()))
            .expect("Only init once");
    }
    driver::timer_enable();
    println!(">> Test user mode!");
    {
        let user_data = loader::get_app_data_by_name("print_A").unwrap();
        let user_thread = thread::Thread::create_root_user_thread(
            user_data,
            "print_A",
            UserAddr::new(ROOT_THREAD_STACK_BASE),
            ROOT_THREAD_STACK_SIZE,
        );
        let user_data_2 = loader::get_app_data_by_name("print_B").unwrap();
        let user_thread_2 = thread::Thread::create_root_user_thread(
            user_data_2,
            "print_B",
            UserAddr::new(ROOT_THREAD_STACK_BASE),
            ROOT_THREAD_STACK_SIZE,
        );
        CURRENT_SCHEDULER
            .try_get()
            .expect("No init")
            .exclusive_access()
            .push_thread(user_thread);
        CURRENT_SCHEDULER
            .try_get()
            .expect("No init")
            .exclusive_access()
            .push_thread(user_thread_2);
        thread::sched();
        panic!("TEST USER MODE NEVER GET TO HERE!!");
    }

    panic!("Close OS!")
}
