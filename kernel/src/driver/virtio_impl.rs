// memory allocate interface

use crate::{frame_allocator::{CURRENT_FRAME_ALLOCATOR, UnsafePageAlloctor}, arch::KERNEL_BASE, frame::{ FrameSize}, addr_type::{PhysAddr, Addr, phys_to_kernel}};




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
    let kaddr=phys_to_kernel(paddr);
    trace!("alloc DMA: kaddr={:#x}, pages={}", kaddr.addr(), pages);
    kaddr.addr() as usize
}

#[no_mangle]
extern "C" fn virtio_dma_dealloc(kaddr: usize, pages: usize) -> i32 {
    let mut t=PhysAddr::new(kaddr as u64);
    for _i in 0..pages{
        CURRENT_FRAME_ALLOCATOR.exclusive_access().unsafe_deallo(t);
        t+=FrameSize::Size4Kb as u64;
   }
   0
}

#[no_mangle]
extern "C" fn virtio_phys_to_virt(paddr: usize) -> usize {
    paddr+KERNEL_BASE as usize
}

#[no_mangle]
extern "C" fn virtio_virt_to_phys(vaddr: usize) -> usize {
    vaddr-KERNEL_BASE as usize
}

