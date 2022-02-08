use core::cell::{RefCell, Cell};

use crate::mm_type::*;
use alloc::vec::Vec;
use crate::up::UPSafeCell;
use lazy_static::*;

/// -------------------------
/// frame allocator interface
/// -------------------------
pub trait FrameAllocator {
    fn allocate_frame(& self) -> Result<PhysFrame, FrameAllocError>;
    fn deallocate_frame(& self, pf: &PhysFrame);
}
#[derive(core::fmt::Debug)]
pub struct FrameAllocError;

lazy_static!{
    pub static ref CurrentFrameAllocator : UPSafeCell<StackFrameAllocator> =unsafe {
        extern "C"{
            fn end();
        }
        let baddr=PhysAddr::from(end as usize-KERNEL_BASE as usize);
        let eaddr=PhysAddr::from(MEMORY_END);
        UPSafeCell::new(StackFrameAllocator::new(PhysPageNum::new(baddr),PhysPageNum::new(eaddr)))
    };
}

/// -----------------------
/// A simple stack allocator
/// ------------------------
#[derive(Debug,Clone)]
pub struct StackFrameAllocator {
    current: Cell<PhysPageNum>,
    end: PhysPageNum,
    recycled: RefCell<Vec<PhysPageNum>>,
}

impl StackFrameAllocator {
    pub fn new(start: PhysPageNum, end: PhysPageNum) -> Self {
        StackFrameAllocator { current: Cell::new(start), end, recycled: RefCell::new(Vec::new()) }
    }
    pub fn allocate(&self) -> Result<PhysFrame, FrameAllocError> {
        if let Some(ppn) = self.recycled.borrow_mut().pop() {
            unsafe{Ok(PhysFrame::new(ppn))}
        } else {
            if self.current.get() == self.end {
                Err(FrameAllocError)
            } else {
                let ppn = self.current.replace(self.current.get().next_page());
                unsafe{Ok(PhysFrame::new(ppn))}
            }
        }
    }
    pub fn deallocate(&self, pf: &PhysFrame) {
        // validity check
        let ppn=pf.ppn();
        if ppn.is_within_range(self.current.get(), self.end) || self.recycled.borrow().iter().find(|&v| {*v == ppn}).is_some() {
            panic!("Frame ppn={:x?} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.borrow_mut().push(ppn);
    }
}

impl FrameAllocator for StackFrameAllocator{
    fn allocate_frame(& self) -> Result<PhysFrame, FrameAllocError> {
        self.allocate()
    }
    fn deallocate_frame(& self, pf: &PhysFrame) {
        self.deallocate(pf)
    }
}

