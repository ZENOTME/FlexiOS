// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2022 Andre Richter <andre.o.richter@gmail.com>

//! Architectural boot code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::boot::arch_boot

use crate::print;

// Assembly counterpart to this file.
core::arch::global_asm!(include_str!("boot.s"));


#[no_mangle]
#[link_section = ".text.boot"]
extern "C" fn clear_bss() {
    let start = sbss as usize;
    let end = ebss as usize;
    let step = core::mem::size_of::<usize>();
    for i in (start..end).step_by(step) {
        unsafe { (i as *mut usize).write(0) };
    }
}

#[link_section = ".text.boot"]
fn map_2mib(p2: &mut PageTable, start: usize, end: usize, flag: EF, attr: Attr) {
    let aligned_start = align_down(start as u64, ALIGN_2MIB);
    let aligned_end = align_up(end as u64, ALIGN_2MIB);
    for frame in PhysFrame::<Size2MiB>::range_of(aligned_start, aligned_end) {
        let paddr = frame.start_address();
        let page = Page::<Size2MiB>::of_addr(phys_to_virt(paddr.as_u64() as usize) as u64);
        p2[page.p2_index()].set_block::<Size2MiB>(paddr, flag, attr);
    }
}

#[no_mangle]
#[link_section = ".text.boot"]
extern "C" fn create_init_paging() {
    let p4 = unsafe { &mut *(page_table_lvl4 as *mut PageTable) };
    let p3 = unsafe { &mut *(page_table_lvl3 as *mut PageTable) };
    let p2 = unsafe { &mut *(page_table_lvl2 as *mut PageTable) };
    let frame_lvl3 = PhysFrame::<Size4KiB>::of_addr(page_table_lvl3 as u64);
    let frame_lvl2 = PhysFrame::<Size4KiB>::of_addr(page_table_lvl2 as u64);
    p4.zero();
    p3.zero();
    p2.zero();

    let block_flags = EF::default_block() | EF::UXN;
    // 0x0000_0000_0000 ~ 0x0080_0000_0000
    p4[0].set_frame(frame_lvl3, EF::default_table(), Attr::new(0, 0, 0));
    // 0x8000_0000_0000 ~ 0x8080_0000_0000
    p4[256].set_frame(frame_lvl3, EF::default_table(), Attr::new(0, 0, 0));

    // 0x0000_0000 ~ 0x4000_0000
    p3[0].set_frame(frame_lvl2, EF::default_table(), Attr::new(0, 0, 0));
    // 0x4000_0000 ~ 0x8000_0000
    p3[1].set_block::<Size1GiB>(
        PhysAddr::new(PERIPHERALS_END as u64),
        block_flags | EF::PXN,
        MairDevice::attr_value(),
    );

    // normal memory (0x0000_0000 ~ 0x3F00_0000)
    map_2mib(
        p2,
        0,
        PERIPHERALS_START,
        block_flags,
        MairNormal::attr_value(),
    );
    // device memory (0x3F00_0000 ~ 0x4000_0000)
    map_2mib(
        p2,
        PERIPHERALS_START,
        align_down(PERIPHERALS_END as u64, ALIGN_1GIB) as usize,
        block_flags | EF::PXN,
        MairDevice::attr_value(),
    );
}


extern "C" {
    fn sbss();
    fn ebss();
    fn page_table_lvl4();
    fn page_table_lvl3();
    fn page_table_lvl2();
    fn _start();
    fn _end();
}