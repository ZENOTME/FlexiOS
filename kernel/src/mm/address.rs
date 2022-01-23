use crate::arch::mm::{PA_WIDTH,VA_WIDTH};
use core::fmt::{self, Debug, Formatter};
use page::PageSize;


/// Definitions
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);


/// Debugging

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

/// From
impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self { Self(v & ( (1 << PA_WIDTH) - 1 )) }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self { Self(v & ( (1 << VA_WIDTH) - 1 )) }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self { v.0 }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self { v.0 }
}

// Address Caculate
impl VirtAddr {
    pub fn floor<U:Into<u64>>(&self,align:U) -> VirtAddr { 
        VirtAddr(self.0 / align.into()) 
    }
    pub fn ceil<U:Into<u64>>(&self,align:U) -> VirtAddr  { 
        VirtAddr((self.0 - 1 + align.into()) / align.into()) 
    }
    pub fn page_offset<U:Into<u64>>(&self,align:U) -> usize { 
        self.0 & (align.into() - 1) 
    }
    pub fn aligned(&self) -> bool { self.page_offset() == 0 }
}

impl PhysAddr {
    pub fn floor<U:Into<u64>>(&self,align:U) -> PhysPageNum { 
        PhysPageNum(self.0 / align.into()) 
    }
    pub fn ceil<U:Into<u64>>(&self,align:U) -> PhysPageNum { 
        PhysPageNum((self.0 - 1 + align.into()) / align.into()) 
    }
    pub fn page_offset<U:Into<u64>>(&self,align:U) -> usize { 
        self.0 & (align.into() - 1) 
    }
    pub fn aligned(&self) -> bool {
         self.page_offset() == 0 
    }
}

