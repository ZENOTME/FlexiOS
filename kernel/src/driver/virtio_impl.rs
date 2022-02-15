// memory allocate interface
use core::{sync::atomic::*, cell::{Ref, RefMut}};
use alloc::vec::Vec;
use lazy_static::lazy_static;
use crate::{frame_allocator::{CurrentFrameAllocator, FrameAllocator}, addr_type::*, arch::KERNEL_BASE, up::UPSafeCell, frame::Frame};


//Store allocator frame
lazy_static!{
    pub static ref VirtioFrameCollector : UPSafeCell<Vec<Frame>> =unsafe {
        UPSafeCell::new(Vec::new())
    };
}



#[no_mangle]
extern "C" fn virtio_dma_alloc(pages: usize) -> usize {
    let phy_frame=CurrentFrameAllocator.exclusive_access().allocate_frames(4096).unwrap()[0];
    VirtioFrameCollector.exclusive_access().push(phy_frame);
    for i in (0..pages-1){
        VirtioFrameCollector.exclusive_access().push(CurrentFrameAllocator.exclusive_access().allocate_frames(4096).unwrap()[0]);
    }
    let paddr;
    if let Frame::Data(data)= phy_frame{
        paddr=data.frame_addr().0
    }
    trace!("alloc DMA: paddr={:#x}, pages={}", paddr, pages);
    paddr
}

#[no_mangle]
extern "C" fn virtio_dma_dealloc(paddr: usize, pages: usize) -> i32 {
    info!("Can't dealloc,may cause memory leakage!");
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

