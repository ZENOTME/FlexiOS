use super::{page_mode::*,address::*,frame::PhysFrame};
use alloc::vec::Vec;
use crate::up::UPSafeCell;
use lazy_static::*;
use crate::arch::board::PERIPHERALS_START;
/// -------------------------
/// frame allocator interface
/// -------------------------
pub trait FrameAllocator {
    fn allocate_frame(&self) -> Result<PhysPageNum, FrameAllocError>;
    fn deallocate_frame(&self, ppn: PhysPageNum);
}
pub struct FrameAllocError;

/// -----------------------
/// FramBox represent the ownership of the frame
/// Use a allocatord to create a frame
/// ------------------------
#[derive(Debug)]
pub struct FrameBox<A: FrameAllocator = StackFrameAllocator,M:PageMode=defaultMode> {
    frame: PhysFrame<M>, // 相当于*mut类型的指针
    frame_alloc: A,
}
impl<A: FrameAllocator> FrameBox<A> {
    // 分配页帧并创建FrameBox
    pub fn try_new_in(frame_alloc: A) -> Result<FrameBox<A>, FrameAllocError> {
        let ppn = frame_alloc.allocate_frame()?;
        Ok(FrameBox { frame: PhysFrame::new(ppn), frame_alloc })
    }
    pub fn phys_page_num(&self) -> PhysPageNum {
        self.frame.ppn()
    }
}
impl<A: FrameAllocator,M: PageMode> Drop for FrameBox<A,M> {
    fn drop(&mut self) {
        // 释放所占有的页帧
        self.frame_alloc.deallocate_frame(self.frame.ppn());
    }
}


/// -----------------------
/// A simple stack allocator
/// ------------------------
#[derive(Debug,Clone)]
pub struct StackFrameAllocator {
    current: PhysPageNum,
    end: PhysPageNum,
    recycled: Vec<PhysPageNum>,
}

impl StackFrameAllocator {
    pub fn new(start: PhysPageNum, end: PhysPageNum) -> Self {
        StackFrameAllocator { current: start, end, recycled: Vec::new() }
    }
    pub fn allocate_frame(&mut self) -> Result<PhysPageNum, FrameAllocError> {
        if let Some(ppn) = self.recycled.pop() {
            Ok(ppn)
        } else {
            if self.current == self.end {
                Err(FrameAllocError)
            } else {
                let ans = self.current;
                self.current = self.current.next_page();
                Ok(ans)
            }
        }
    }
    pub fn deallocate_frame(&mut self, ppn: PhysPageNum) {
        // validity check
        if ppn.is_within_range(self.current, self.end) || self.recycled.iter().find(|&v| {*v == ppn}).is_some() {
            panic!("Frame ppn={:x?} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

impl FrameAllocator for StackFrameAllocator{
    fn allocate_frame(&self) -> Result<PhysPageNum, FrameAllocError> {
        self.allocate_frame()
    }
    fn deallocate_frame(&self, ppn: PhysPageNum) {
        self.deallocate_frame(ppn)
    }
}

lazy_static!{
    pub static ref DefaultFrameAllocator : UPSafeCell<StackFrameAllocator> =unsafe {
        extern "C"{
            fn ekernel();
        }
        let baddr=PhysAddr::from(ekernel as usize);
        let eaddr=PhysAddr::from(PERIPHERALS_START);
        UPSafeCell::new(StackFrameAllocator::new(PhysPageNum::new::<defaultMode>(baddr),PhysPageNum::new::<defaultMode>(eaddr)))
    };
}