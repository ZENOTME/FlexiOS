use core::{cell::{RefCell, Cell}, cmp};


use crate::{addr_type::*, frame::*};
use alloc::vec::Vec;
use crate::up::UPSafeCell;
use lazy_static::*;

/// -------------------------
/// frame allocator interface
/// -------------------------
pub trait FrameAllocator {
    fn create_allocator(baddr:PhysAddr,eaddr:PhysAddr)->Self;
    fn allocate_frames(& self,size:usize) -> Result<Vec<Frame>, FrameAllocError>;
    fn deallocate_frame(& self, pf: &DataFrame);
}

#[derive(core::fmt::Debug)]
pub struct FrameAllocError;

type CurrentFrameAllocatorType=StackFrameAllocator;

lazy_static!{
    pub static ref CurrentFrameAllocator : UPSafeCell<CurrentFrameAllocatorType> =unsafe {
        extern "C"{
            fn end();
        }
        let baddr=PhysAddr::from(end as usize-KERNEL_BASE as usize);
        let eaddr=PhysAddr::from(MEMORY_END);
        UPSafeCell::new(StackFrameAllocator::create_allocator(baddr,eaddr))
    };
}

/// -----------------------
/// A simple stack allocator
/// recycled
/// 0 4Mb
/// 1 2Mb
/// 2 1Gb
/// -----------------------
#[derive(Debug,Clone)]
pub struct StackFrameAllocator {
    current: Cell<PhysAddr>,
    end: PhysAddr,
    recycled: [RefCell<Vec<PhysAddr>>;3],
}


impl FrameAllocator for StackFrameAllocator{
    fn create_allocator(baddr:PhysAddr,eaddr:PhysAddr)->Self {
       Self{ 
           current: Cell::new(baddr),
           end: eaddr,
           recycled:[RefCell::new(Vec::new());3]
        }
    }
    
    fn allocate_frames(& self,size:usize) -> Result<Vec<Frame>, FrameAllocError> {
        let unalloc_size=size;
        let frames:Vec<Frame>=Vec::new();
        // HardCode
        // 0-4096*256
        // 4096*256-4096*512*256
        // 4096*512*256-other
        let _1_level:usize=4096*256;
        let _2_level:usize=4096*512*256;
    
        while unalloc_size!=0 {
            let index:i32;
            let sz;
            match unalloc_size{
                0..=_1_level=>{
                    index=0;
                    sz=FrameSize::Size4Kb;
                }
                _1_level..=_2_level=>{
                    index=1;
                    sz=FrameSize::Size2Mb;
                }
                other=>{
                    index=2;
                    sz=FrameSize::Size1Gb;
                }
            }
            if let Some(pa) = self.recycled[index as usize].borrow_mut().pop(){
                frames.push(Frame::Data(DataFrame{
                    start:pa,
                    size:sz
                }));
            }else{
                if self.current.get()+sz as usize > self.end {
                    loop{
                        index-=1;
                        if index<0 { break };
                        match index {
                            0=>{
                                if self.current.get()+FrameSize::Size4Kb as usize <= self.end {
                                    sz=FrameSize::Size4Kb;
                                    break;
                                }
                            }
                            1=>{
                                if self.current.get()+FrameSize::Size2Mb as usize <= self.end {
                                    sz=FrameSize::Size2Mb;
                                    break;
                                }
                            }
                            _=>{
                                return Err(FrameAllocError);
                            }
                        }
                    }
                    if index<0 {return Err(FrameAllocError)};
                } 
                
                let pa = self.current.replace(self.current.get()+sz as usize);
                frames.push(Frame::Data(DataFrame{
                    start:pa,
                    size:sz
                }));
            }
            unalloc_size -= cmp::min(unalloc_size,sz as usize)
        }
        Ok(frames)
    }

    fn deallocate_frame(& self, df: &DataFrame) {
        match df.frame_size(){
            FrameSize::Size4Kb => {
                self.recycled[0].borrow_mut().push(df.frame_addr())
            },
            FrameSize::Size2Mb => {
                self.recycled[1].borrow_mut().push(df.frame_addr())
            },
            FrameSize::Size1Gb => {
                self.recycled[2].borrow_mut().push(df.frame_addr())
            }
        }
    }
}

