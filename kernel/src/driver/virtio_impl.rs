// memory allocate interface

use alloc::vec::Vec;
use lazy_static::lazy_static;
use crate::{frame_allocator::{CURRENT_FRAME_ALLOCATOR, FrameAllocator, UnsafePageAlloctor}, arch::KERNEL_BASE, up::UPSafeCell, frame::{DataFrame, FrameSize}, addr_type::PhysAddr};




#[no_mangle]
extern "C" fn virtio_dma_alloc(pages: usize) -> usize {
    let mut paddr=None;
    for _i in 0..pages {
        match paddr{
            Some(_)=>{CURRENT_FRAME_ALLOCATOR.exclusive_access().unsafe_alloc_page().unwrap();},
            None=>{
                let t=CURRENT_FRAME_ALLOCATOR.exclusive_access().unsafe_alloc_page().unwrap();
                paddr=Some(t);
            }
        }
    }
    let paddr=paddr.unwrap();
    trace!("alloc DMA: paddr={:#x}, pages={}", paddr.0, pages);
    paddr.0
}

#[no_mangle]
extern "C" fn virtio_dma_dealloc(paddr: usize, pages: usize) -> i32 {
    let mut t=paddr;
    for _i in 0..pages{
        CURRENT_FRAME_ALLOCATOR.exclusive_access().unsafe_deallo(PhysAddr::from(t));
        t+=FrameSize::Size4Kb as usize;
   }
   0
}

#[no_mangle]
extern "C" fn virtio_phys_to_virt(paddr: usize) -> usize {
    paddr+KERNEL_BASE
}

#[no_mangle]
extern "C" fn virtio_virt_to_phys(vaddr: usize) -> usize {
    vaddr-KERNEL_BASE
}

