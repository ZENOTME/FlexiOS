use crate::mm_type::*;
use alloc::vec::Vec;
use crate::up::UPSafeCell;
use lazy_static::*;

/// -------------------------
/// frame allocator interface
/// -------------------------
pub trait FrameAllocator {
    fn allocate_frame(&self) -> Result<PhysFrame, FrameAllocError>;
    fn deallocate_frame(&self, pf: PhysFrame);
}
pub struct FrameAllocError;

lazy_static!{
    pub static ref CurrentFrameAllocator : UPSafeCell<StackFrameAllocator> =unsafe {
        extern "C"{
            fn ekernel();
        }
        let baddr=PhysAddr::from(ekernel as usize);
        let eaddr=PhysAddr::from(MEMORY_END);
        UPSafeCell::new(StackFrameAllocator::new(PhysPageNum::new(baddr),PhysPageNum::new(eaddr)))
    };
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
    pub fn allocate_frame(&mut self) -> Result<PhysFrame, FrameAllocError> {
        if let Some(ppn) = self.recycled.pop() {
            unsafe{Ok(PhysFrame::new(ppn))}
        } else {
            if self.current == self.end {
                Err(FrameAllocError)
            } else {
                let ppn = self.current;
                self.current = self.current.next_page();
                unsafe{Ok(PhysFrame::new(ppn))}
            }
        }
    }
    pub fn deallocate_frame(&mut self, pf: &PhysFrame) {
        // validity check
        let ppn=pf.ppn();
        if ppn.is_within_range(self.current, self.end) || self.recycled.iter().find(|&v| {*v == ppn}).is_some() {
            panic!("Frame ppn={:x?} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

impl FrameAllocator for StackFrameAllocator{
    fn allocate_frame(&self) -> Result<PhysFrame, FrameAllocError> {
        self.allocate_frame()
    }
    fn deallocate_frame(&self, pf: PhysFrame) {
        self.deallocate_frame(pf)
    }
}

