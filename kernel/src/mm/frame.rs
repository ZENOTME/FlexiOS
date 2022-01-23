//! Abstractions for default-sized and huge physical memory frames.

use crate::{
    PhysAddr,
};
use core::{
    fmt,
    marker::PhantomData,
    ops::{Add, AddAssign, Sub, SubAssign},
};

/// A physical memory frame.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct PhysFrame<S: PageSize = Size4KiB> {
    start_address: PhysAddr,
    size: PhantomData<S>,
}

impl <S:PageSize> PhysFrame<S>{
    pub fn from_start_address(address: PhysAddr) -> Result<Self, ()> {
        if !address.is_aligned(S::SIZE) {
            return Err(());
        }
        Ok(PhysFrame::containing_address(address))
    }
    /// Returns the frame that contains the given physical address.
    pub fn containing_address(address: PhysAddr) -> Self {
        PhysFrame {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }
    /// Returns the start address of the frame.
    pub fn start_address(&self) -> PhysAddr {
        self.start_address
    }

    /// Returns the size the frame (4KB, 2MB or 1GB).
    pub fn size(&self) -> u64 {
        S::SIZE
    }

}