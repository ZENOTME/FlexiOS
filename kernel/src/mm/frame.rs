//! Abstractions for default-sized and huge physical memory frames.

use crate::mm::{
    address::PhysAddr,
    page::PageSize,
};
use crate::arch::paging::Size4KiB;
use core::{
    fmt,
    marker::PhantomData,
    ops::{Add, AddAssign, Sub, SubAssign},
};

/// A physical memory frame.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct PhysFrame<S: PageSize=Size4KiB> {
    start_address: PhysAddr,
    size: PhantomData<S>,
}

impl <S:PageSize> PhysFrame<S>{
    pub fn from_start_address(address: PhysAddr) -> Result<Self, ()> {
        if !address.aligned(S::SIZE) {
            return Err(());
        }
        Ok(PhysFrame::containing_address(address))
    }
    /// Returns the frame that contains the given physical address.
    pub fn containing_address(address: PhysAddr) -> Self {
        PhysFrame {
            start_address: address.floor(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns the start address of the frame.
    pub fn start_address(&self) -> PhysAddr {
        self.start_address
    }

    /// Returns the size the frame (4KB, 2MB or 1GB).
    pub fn size(&self) -> u64 {
        S::SIZE as u64
    }

    /// Returns a range of frames, exclusive `end`.
    pub fn range(start: PhysFrame<S>, end: PhysFrame<S>) -> PhysFrameRange<S> {
        PhysFrameRange { start, end }
    }

    pub fn range_of(begin: PhysAddr, end: PhysAddr) -> PhysFrameRange<S> {
        Self::range(Self::containing_address(begin), Self::containing_address(end - 1) + 1)
    }

}

impl<S: PageSize> Add<usize> for PhysFrame<S> {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        PhysFrame::containing_address(self.start_address() + rhs * usize::from(S::SIZE))
    }
}

impl<S: PageSize> AddAssign<usize> for PhysFrame<S> {
    fn add_assign(&mut self, rhs: usize) {
        *self = self.clone() + rhs;
    }
}

/// An range of physical memory frames, exclusive the upper bound.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct PhysFrameRange<S: PageSize = Size4KiB> {
    /// The start of the range, inclusive.
    pub start: PhysFrame<S>,
    /// The end of the range, exclusive.
    pub end: PhysFrame<S>,
}

impl<S: PageSize> PhysFrameRange<S> {
    /// Returns whether the range contains no frames.
    pub fn is_empty(&self) -> bool {
        !(self.start < self.end)
    }
}

impl<S: PageSize> Iterator for PhysFrameRange<S> {
    type Item = PhysFrame<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let frame = self.start.clone();
            self.start += 1;
            Some(frame)
        } else {
            None
        }
    }
}

impl<S: PageSize> fmt::Debug for PhysFrame<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "PhysFrame[{}]({:#x})",
            S::SIZE_AS_DEBUG_STR,
            usize::from(self.start_address())
        ))
    }
}

impl<S: PageSize> fmt::Debug for PhysFrameRange<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PhysFrameRange")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}