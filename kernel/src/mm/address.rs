use crate::arch::mm::{PA_WIDTH,VA_WIDTH};
use core::fmt::{self, Debug, Formatter};
use core::ops::{Add,AddAssign,Sub,SubAssign};

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