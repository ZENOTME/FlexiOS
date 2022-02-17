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

use crate::{arch::paging::{
    page_table::{PageTable,PageTableFlags},
}, addr_type::*};
use crate::arch::board::*;
use cortex_a::{registers::*,asm::barrier};
use tock_registers::interfaces::{Writeable,Readable,ReadWriteable};



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



#[no_mangle]
#[link_section = ".text.boot"]
extern "C" fn create_init_paging() {
    let p0 = unsafe { &mut *(page_table_lvl0 as *mut PageTable) };
    let p1 = unsafe { &mut *(page_table_lvl1 as *mut PageTable) };
    let ppn_lvl1:PhysAddr;
    unsafe{
        ppn_lvl1 = PhysAddr::from(page_table_lvl1 as usize);
    }
    p0.zero();
    p1.zero();

    let block_flags =  PageTableFlags::UXN::SET;
    // 0x0000_0000_0000 ~ 0x0080_0000_0000
    p0[0].set_table_page(ppn_lvl1, None);
    // 0x8000_0000_0000 ~ 0x8080_0000_0000
    p0[256].set_table_page(ppn_lvl1, None);
    // 0x0000_0000 ~ 0x4000_0000
    p1[1].set_huge_page(PhysAddr::from(MEMORY_START as usize), 
        Some(block_flags+PageTableFlags::SH::INNERSHARE+PageTableFlags::ATTR_INDEX.val(0)+PageTableFlags::AF::SET)
    );
    // 0x4000_0000 ~ 0x8000_0000
    p1[0].set_huge_page(
        PhysAddr::from(PERIPHERALS_START as usize),
        Some(block_flags + PageTableFlags::PXN::SET+PageTableFlags::SH::OUTERSHARE+PageTableFlags::ATTR_INDEX.val(1)+PageTableFlags::AF::SET)
    );
}

#[no_mangle]
#[link_section = ".text.boot"]
extern "C" fn enable_mmu() {
    //Memory Config
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc+MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_EarlyWriteAck
            + MAIR_EL1::Attr2_Normal_Outer::NonCacheable+MAIR_EL1::Attr2_Normal_Inner::NonCacheable,
    );

    // Configure various settings of stage 1 of the EL1 translation regime.
    let ips = ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange); //->Physicall Address Range
    TCR_EL1.write(
        TCR_EL1::TBI1::Ignored
            + TCR_EL1::TBI0::Ignored
            + TCR_EL1::AS::ASID16Bits                                     //->ASID 16 bit
            + TCR_EL1::IPS.val(ips)
            + TCR_EL1::TG1::KiB_4                                         //->Granule size for the TTBR1_RL1
            + TCR_EL1::SH1::Inner                                         //Shareability
            + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable    //Cachebility
            + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::EPD1::EnableTTBR1Walks                             //Enable TTBR
            + TCR_EL1::A1::TTBR0                                          //?? what
            + TCR_EL1::T1SZ.val(16)
            + TCR_EL1::TG0::KiB_4
            + TCR_EL1::SH0::Inner
            + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::T0SZ.val(16),
    );

    // Set both TTBR0_EL1 and TTBR1_EL1
    let paddr = page_table_lvl0 as u64;
    TTBR0_EL1.set_baddr(paddr);
    TTBR1_EL1.set_baddr(paddr);
    unsafe {
        llvm_asm!(
            "dsb ishst
             tlbi vmalle1is
             dsb ish
             isb"
            :::: "volatile"
        );
    }
    //unsafe{barrier::isb(barrier::SY);}
    // Enable the MMU and turn on data and instruction caching.
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable+
        SCTLR_EL1::A::CLEAR+SCTLR_EL1::SA::CLEAR+SCTLR_EL1::SA0::CLEAR+SCTLR_EL1::NAA::CLEAR);
    
    // Force MMU init to complete before next instruction
    unsafe{barrier::isb(barrier::SY);}

    // Invalidate the local I-cache so that any instructions fetched
    // speculatively from the PoC are discarded
    unsafe { llvm_asm!("ic iallu; dsb nsh; isb":::: "volatile") };
}


extern "C" {
    fn sbss();
    fn ebss();
    fn page_table_lvl0();
    fn page_table_lvl1();
    fn _start();
    fn _end();
}