// memory allocate interface
use core::{sync::atomic::*, cell::{Ref, RefMut}};
use alloc::vec::Vec;
use lazy_static::lazy_static;
use crate::{frame_allocator::{CurrentFrameAllocator, FrameAllocator}, mm_type::PhysFrame};


#[no_mangle]
extern "C" fn virtio_dma_alloc(pages: usize) -> usize {
    let mut vframe:Vec<PhysFrame>=Vec::new();
    //let allocator=CurrentFrameAllocator.exclusive_access();
    let phy_frame=CurrentFrameAllocator.exclusive_access().allocate_frame().unwrap();
    info!("!!!");
    for i in (0..pages-1){
        vframe.push(CurrentFrameAllocator.exclusive_access().allocate_frame().unwrap());
        info!("!!!");
    }
    let paddr = phy_frame.ppn().addr().0;
    trace!("alloc DMA: paddr={:#x}, pages={}", paddr, pages);
    paddr
}

#[no_mangle]
extern "C" fn virtio_dma_dealloc(paddr: usize, pages: usize) -> i32 {
    trace!("dealloc DMA: paddr={:#x}, pages={}", paddr, pages);
    0
}

#[no_mangle]
extern "C" fn virtio_phys_to_virt(paddr: usize) -> usize {
    paddr+0xffff_0000_0000_0000
}

#[no_mangle]
extern "C" fn virtio_virt_to_phys(vaddr: usize) -> usize {
    vaddr-0xffff_0000_0000_0000
}

