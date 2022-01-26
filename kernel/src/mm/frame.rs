//! Abstractions for default-sized and huge physical memory frames.

use crate::mm::{
    address::PhysAddr,
};

use core::{
    marker::PhantomData,
    ops::{Range},
};
use super::page_mode::*;

/// A physical memory frame.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct PhysFrame<M: PageMode=defaultMode> {
    ppn : PhysPageNum,
    size: PhantomData<M>,
}

impl <M:PageMode> PhysFrame<M>{
    /// Returns the frame that contains the given physical address.
    /// Unsafe! This operation obly used for boot or sepical frame which are alloc in advance
    pub unsafe fn containing_address(address: PhysAddr) -> Self {
        Self {
            ppn: PhysPageNum::new::<M>(address),
            size: PhantomData,
        }
    }
    pub unsafe fn range_of(begin: PhysAddr, end: PhysAddr) -> PhysFrameRange<M> {
        Self::range(Self::containing_address(begin), Self::containing_address(end + (1<<M::FRAME_SIZE_BITS)-1))
    }
    /// Normal crate
    pub fn new(_ppn:PhysPageNum)->Self{
        Self{
            ppn: _ppn,
            size: PhantomData,
        }
    }

    /// Returns the start address of the frame.
    pub fn ppn(&self) -> PhysPageNum {
        self.ppn
    }

    /// Returns the size the frame (4KB, 16KB).
    pub const fn size(&self) -> u64 {
        (1<<M::FRAME_SIZE_BITS) as u64
    }

    /// Returns a range of frames, exclusive `end`.
    pub fn range(start: PhysFrame<M>, end: PhysFrame<M>) -> PhysFrameRange<M> {
        PhysFrameRange { start, end }
    }

}


/// An range of physical memory frames, exclusive the upper bound.
#[derive(Clone, PartialEq, Eq)]
#[repr(C)]
pub struct PhysFrameRange<M:PageMode = defaultMode> {
    /// The start of the range, inclusive.
    pub start: PhysFrame<M>,
    /// The end of the range, exclusive.
    pub end: PhysFrame<M>,
}

impl<M: PageMode> PhysFrameRange<M> {
    /// Returns whether the range contains no frames.
    pub fn is_empty(&self) -> bool {
        !(self.start.ppn() < self.end.ppn())
    }
    pub fn range(&self) -> Range<usize>{
        let start=self.start.ppn().0;
        let end=self.end.ppn().0;
        start..end
    }
}
