use core::fmt::{self, Debug, Formatter};
use core::ops::{Add,AddAssign,Sub,SubAssign};
use crate::frame_allocator::{CurrentFrameAllocator, FrameAllocator};
/// Re-export the mm_type const
pub use crate::arch::mm_type::{
    PAGE_SIZE,PAGE_SIZE_BITS,PA_WIDTH,VA_WIDTH,KERNEL_BASE
};
pub use crate::arch::board::MEMORY_END;
pub use crate::arch::paging::PageTableFlags;

/// ---------------
/// Addr Definitions
/// ---------------
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
    pub fn floor<U:Into<usize>>(&self,align:U) -> VirtAddr { 
        VirtAddr(floor(self.0,align.into()))
    }
    pub fn ceil<U:Into<usize>>(&self,align:U) -> VirtAddr  { 
        VirtAddr(ceil(self.0,align.into()))
    }
    pub fn page_offset<U:Into<usize>>(&self,align:U) -> usize { 
        self.0 & (align.into() - 1) 
    }
    pub fn aligned<U:Into<usize>>(&self,align:U) -> bool { 
        self.page_offset(align) == 0 
    }
}

impl PhysAddr {
    pub fn floor<U:Into<usize>>(&self,align:U) -> PhysAddr { 
        PhysAddr(floor(self.0,align.into()))
    }
    pub fn ceil<U:Into<usize>>(&self,align:U) -> PhysAddr { 
        PhysAddr(ceil(self.0,align.into()))
    }
    pub fn page_offset<U:Into<usize>>(&self,align:U) -> usize { 
        self.0 & (align.into() - 1) 
    }
    pub fn aligned<U:Into<usize>>(&self,align:U) -> bool { 
        self.page_offset(align) == 0 
    }
}

#[inline]
pub fn floor(addr:usize,align:usize)->usize{
    debug_assert!(align.is_power_of_two(), "`align` must be a power of two");
    addr & !(align-1)
}
#[inline]
pub fn ceil(addr: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr // already aligned
    } else {
        (addr | align_mask) + 1
    }
}


impl Add<usize> for VirtAddr {
    type Output = Self;
    fn add(self, rhs: usize) -> VirtAddr {
        VirtAddr::from(self.0 + rhs)
    }
}

impl AddAssign<usize> for VirtAddr {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Sub<usize> for VirtAddr {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        VirtAddr::from(self.0.checked_sub(rhs).unwrap())
    }
}

impl SubAssign<usize> for VirtAddr {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}


impl Add<usize> for PhysAddr {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        PhysAddr::from(self.0 + rhs)
    }
}

impl AddAssign<usize> for PhysAddr {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Sub<usize> for PhysAddr {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        PhysAddr::from(self.0.checked_sub(rhs).unwrap())
    }
}

impl SubAssign<usize> for PhysAddr {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

pub fn phys_to_virt(paddr:PhysAddr)->VirtAddr{
    let t=usize::from(paddr);
    VirtAddr(t.checked_add(KERNEL_BASE).unwrap())
}
pub fn virt_to_phys(vaddr:VirtAddr)->PhysAddr{
    let t=usize::from(vaddr);
    PhysAddr(t.checked_sub(KERNEL_BASE).unwrap())
}

/// ------------
/// PageNum Definition
/// ------------
#[derive(Copy, Clone, PartialEq,PartialOrd, Eq, Debug,Ord)]
pub struct PhysPageNum(pub usize);

impl PhysPageNum {
    pub fn addr(&self) -> PhysAddr {
        PhysAddr(self.0 << PAGE_SIZE_BITS)
    }
    pub fn new(paddr:PhysAddr)->Self{
        PhysPageNum(paddr.0>>PAGE_SIZE_BITS)
    }
    pub fn next_page(&self) -> PhysPageNum {
        // PhysPageNum不处理具体架构的PPN_BITS，它的合法性由具体架构保证
        PhysPageNum(self.0.wrapping_add(1))
    }
    pub fn is_within_range(&self, begin: PhysPageNum, end: PhysPageNum) -> bool {
        if begin.0 <= end.0 {
            begin.0 <= self.0 && self.0 < end.0
        } else {
            begin.0 <= self.0 || self.0 < end.0
        }
    }
}

#[derive(Copy, Clone, PartialEq,PartialOrd, Eq, Debug,Ord)]
pub struct VirtPageNum(pub usize);

impl VirtPageNum {
    pub fn addr(&self) -> VirtPageNum {
        VirtPageNum(self.0 << PAGE_SIZE_BITS)
    }
    pub fn new(vaddr:VirtAddr)->Self{
        Self(vaddr.0>>PAGE_SIZE_BITS)
    }
}

/// ----------------------
/// A physical memory frame.
/// -----------------------
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord,Debug)]
#[repr(C)]
pub struct PhysFrame {
    ppn : PhysPageNum
}

impl PhysFrame{
    /// Returns the frame that contains the given physical address.
    /// Unsafe! This operation obly used for boot or sepical frame which are alloc in advance
    pub unsafe fn containing_address(address: PhysAddr) -> Self {
        Self {
            ppn: PhysPageNum::new(address)
        }
    }
    pub unsafe fn new(_ppn:PhysPageNum)->Self{
        Self{
            ppn: _ppn,
        }
    }
    /// Returns the start address of the frame.
    pub fn ppn(&self) -> PhysPageNum {
        self.ppn
    }
}

impl Drop for PhysFrame {
    fn drop(&mut self) {
        // 释放所占有的页帧
        CurrentFrameAllocator.exclusive_access().deallocate_frame(self);
    }
}

