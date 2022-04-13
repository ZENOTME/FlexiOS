// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! A panic handler that infinitely waits.

use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo)->!{
    if let Some(location)=_info.location(){
        println!(
            "Panicked at{}:{} {}",
            location.file(),
            location.line(),
            _info.message().unwrap()
        );
    }else{
        println!("Panicked: {}",_info.message().unwrap());
    }
    crate::arch::cpu::wait_forever()
}
