
use crate::mm::address::*;
use core::{
    fmt,
    marker::PhantomData,
    ops::{Add, AddAssign, Sub, SubAssign},
};

pub trait PageSize: Copy + Eq + PartialOrd + Ord {
    /// The page size in bytes.
    const SIZE: usize;
   /// A string representation of the page size for debug output.
    const SIZE_AS_DEBUG_STR: &'static str;
}


/* ====================================Page Size=========================================== */
/// A standard 4KiB page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size4KiB {}

/// A “huge” 2MiB page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size2MiB {}

/// A “giant” 1GiB page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size1GiB {}

impl PageSize for Size4KiB {
    const SIZE: usize = 4096;
    const SIZE_AS_DEBUG_STR: &'static str = "4KiB";
}

impl PageSize for Size2MiB {
    const SIZE: usize = Size4KiB::SIZE * 512;
    const SIZE_AS_DEBUG_STR: &'static str = "2MiB";
}

impl PageSize for Size1GiB {
    const SIZE: usize = Size2MiB::SIZE * 512;
    const SIZE_AS_DEBUG_STR: &'static str = "1GiB";
}

/// A virtual memory page.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Page<S: PageSize = Size4KiB> {
    start_address: VirtAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    // The page size in byte
    pub const SIZE: usize= S::SIZE;
    /// Returns the page that contains the given virtual address.
    pub fn containing_address(address: VirtAddr) -> Self {
        Page {
            start_address: address.floor(S::SIZE),
            size: PhantomData,
        }
    }
    /// Returns the page that starts at the given virtual address.
    ///
    /// Returns an error if the address is not correctly aligned (i.e. is not a valid page start).
    pub fn from_start_address(address: VirtAddr) -> Result<Self, ()> {
        if !address.aligned(S::SIZE) {
            return Err(());
        }
        Ok(Page::containing_address(address))
    }
    /// Returns the start address of the page.
    pub fn start_address(&self) -> VirtAddr {
        self.start_address
    }
    /// Returns the size the page (4KB, 2MB or 1GB).
    pub const fn size(&self) -> usize {
        S::SIZE
    }
    pub fn p0_index(&self)->usize{
        let t =usize::from(self.start_address());
        (t>>12>>9>>9>>9)&0x1ff
    }
    pub fn p1_index(&self)->usize{
        let t =usize::from(self.start_address());
        (t>>12>>9>>9)&0x1ff
    }
}
impl Page<Size2MiB> {
    pub fn p2_index(&self)->usize{
        let t =usize::from(self.start_address());
        (t>>12>>9)&0x1ff
    }   
}
impl Page<Size4KiB> {
    pub fn p2_index(&self)->usize{
        let t =usize::from(self.start_address());
        (t>>12>>9)&0x1ff
    }  
    pub fn p3_index(&self)->usize{
        let t =usize::from(self.start_address());
        (t>>12)&0x1ff
    }     
}